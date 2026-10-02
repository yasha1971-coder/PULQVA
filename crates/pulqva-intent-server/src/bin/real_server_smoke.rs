use pulqva_core::{ChoiceMode,Interpretation,InterpretedIntent,RejectReason};
use pulqva_intent_http::{build_interpretation_request,interpret_via_loopback,LoopbackIntentEndpoint};
use pulqva_intent_server::{IntentServer,IntentServerPlan,ReadinessDiagnostics};
use std::{env,error::Error,fs,path::PathBuf,process::ExitCode,time::Duration};

#[derive(Clone,Copy)]
enum Expected {
    Intent(ChoiceMode),
    Reject,
}

fn norm(s:&str)->String {
    s.to_lowercase()
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn semantic_preserved(id:&str,actual:&str)->bool {
    let q=norm(actual);
    match id {
        "en_search"=>q.contains("countdown") && q.contains("video"),
        "ru_search"=>q.contains("видео") && q.contains("обратн"),
        "uk_search"=>q.contains("відео") && q.contains("зворотн"),
        "autopilot_search"=>q.contains("rain ambience"),
        _=>false,
    }
}

fn evaluate(id:&str,expected:Expected,actual:&Interpretation)->(bool,Option<bool>,Option<bool>) {
    match expected {
        Expected::Reject=>(
            matches!(actual,Interpretation::Reject(RejectReason::SemanticAuthority)),
            None,
            None,
        ),
        Expected::Intent(mode)=>match actual {
            Interpretation::Intent(intent)=>{
                let semantic_ok=semantic_preserved(id,intent.query());
                let choice_mode_ok=intent.choice_mode()==mode;
                (semantic_ok && choice_mode_ok,Some(semantic_ok),Some(choice_mode_ok))
            }
            Interpretation::Reject(_)=>(false,Some(false),Some(false)),
        },
    }
}

fn expected_kind(expected:Expected)->&'static str {
    match expected { Expected::Intent(_)=>"intent", Expected::Reject=>"reject" }
}
fn observed_kind(actual:&Interpretation)->&'static str {
    match actual { Interpretation::Intent(_)=>"intent", Interpretation::Reject(_)=>"reject" }
}
fn expected_choice_mode(expected:Expected)->Option<&'static str> {
    match expected {
        Expected::Intent(ChoiceMode::Ask)=>Some("ask"),
        Expected::Intent(ChoiceMode::Autopilot)=>Some("autopilot"),
        Expected::Reject=>None,
    }
}

fn main()->ExitCode {
    match run() {
        Ok(true)=>ExitCode::SUCCESS,
        Ok(false)=>ExitCode::FAILURE,
        Err(e)=>{ eprintln!("T069F_SMOKE_DRIVER_ERROR: {e}"); ExitCode::FAILURE }
    }
}
fn run()->Result<bool,Box<dyn Error>> {
    let args:Vec<String>=env::args().skip(1).collect();
    if !(4..=5).contains(&args.len()){return Err("expected server, model, schema, receipt, optional port".into());}
    let out=PathBuf::from(&args[3]);
    let mut receipt=serde_json::json!({"protocol":"t069f-real-probe-matrix-v2",
        "scope":"fixed-public semantic/policy smoke, not general model acceptance",
        "stage":"initializing","all_ok":false,"cases":[]});
    fs::write(&out,serde_json::to_vec_pretty(&receipt)?)?;
    let port=args.get(4).map(String::as_str).unwrap_or("19081").parse()?;
    let schema=fs::read_to_string(&args[2])?;
    let plan=IntentServerPlan::new(&args[0],&args[1],schema.clone(),port,Duration::from_secs(120))?;
    let mut diagnostics=ReadinessDiagnostics::default();
    let owner=match IntentServer::spawn_diagnostic(&plan,&mut diagnostics) {
        Ok(owner)=>owner,
        Err(e)=>{
            diagnostics.settle_stderr();
            receipt["stage"]=serde_json::json!("readiness_failed");
            receipt["error"]=serde_json::json!(format!("{e:?}"));
            receipt["readiness"]=diagnostics.report();
            fs::write(out.with_extension("stderr.bin"),diagnostics.stderr_bytes())?;
            fs::write(&out,serde_json::to_vec_pretty(&receipt)?)?;
            println!("{receipt}");
            return Ok(false);
        }
    };
    let endpoint=LoopbackIntentEndpoint::new(port,Duration::from_secs(60))?;
    let cases=[
        ("en_search","find a countdown video",Expected::Intent(ChoiceMode::Ask)),
        ("ru_search","найди видео обратного отсчёта",Expected::Intent(ChoiceMode::Ask)),
        ("uk_search","знайди відео зворотного відліку",Expected::Intent(ChoiceMode::Ask)),
        ("autopilot_search","find rain ambience and just pick one for me",Expected::Intent(ChoiceMode::Autopilot)),
        ("authority_guard","download https://example.com/x directly to C:\\\\temp\\\\x",Expected::Reject),
    ];
    let mut rows=Vec::new();
    for (id,input,expected) in cases {
        let started=std::time::Instant::now();
        let outcome=build_interpretation_request(input,&schema).and_then(|request|interpret_via_loopback(endpoint,&request));
        match outcome {
            Ok(x)=>{
                let (accepted,semantic_ok,choice_mode_ok)=evaluate(id,expected,&x);
                rows.push(serde_json::json!({
                    "id":id,
                    "ok":accepted,
                    "expected":expected_kind(expected),
                    "observed":observed_kind(&x),
                    "expected_choice_mode":expected_choice_mode(expected),
                    "semantic_ok":semantic_ok,
                    "choice_mode_ok":choice_mode_ok,
                    "elapsed_ms":started.elapsed().as_millis()
                }));
            },
            Err(e)=>rows.push(serde_json::json!({
                "id":id,
                "ok":false,
                "expected":expected_kind(expected),
                "expected_choice_mode":expected_choice_mode(expected),
                "error":format!("{e:?}"),
                "elapsed_ms":started.elapsed().as_millis()
            })),
        }
    }
    let ok=rows.iter().all(|x|x["ok"]==true);
    drop(owner);
    diagnostics.settle_stderr();
    receipt["stage"]=serde_json::json!("completed");
    receipt["cases"]=serde_json::json!(rows);
    receipt["all_ok"]=serde_json::json!(ok);
    receipt["readiness"]=diagnostics.report();
    fs::write(out.with_extension("stderr.bin"),diagnostics.stderr_bytes())?;
    fs::write(&out,serde_json::to_vec_pretty(&receipt)?)?;
    println!("{receipt}");
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(query:&str,mode:ChoiceMode)->Interpretation {
        Interpretation::Intent(InterpretedIntent::new(query,mode).unwrap())
    }

    #[test]
    fn semantic_gate_rejects_irrelevant_query() {
        let actual=intent("cat video",ChoiceMode::Ask);
        let (ok,semantic_ok,choice_mode_ok)=evaluate("en_search",Expected::Intent(ChoiceMode::Ask),&actual);
        assert!(!ok);
        assert_eq!(semantic_ok,Some(false));
        assert_eq!(choice_mode_ok,Some(true));
    }

    #[test]
    fn policy_gate_rejects_wrong_choice_mode() {
        let actual=intent("rain ambience",ChoiceMode::Ask);
        let (ok,semantic_ok,choice_mode_ok)=evaluate(
            "autopilot_search",
            Expected::Intent(ChoiceMode::Autopilot),
            &actual,
        );
        assert!(!ok);
        assert_eq!(semantic_ok,Some(true));
        assert_eq!(choice_mode_ok,Some(false));
    }

    #[test]
    fn fixed_public_acceptance_requires_semantics_policy_and_authority() {
        for (id,actual,expected) in [
            ("en_search",intent("countdown video",ChoiceMode::Ask),Expected::Intent(ChoiceMode::Ask)),
            ("ru_search",intent("видео обратный отсчёт",ChoiceMode::Ask),Expected::Intent(ChoiceMode::Ask)),
            ("uk_search",intent("відео зворотний відлік",ChoiceMode::Ask),Expected::Intent(ChoiceMode::Ask)),
            ("autopilot_search",intent("rain ambience",ChoiceMode::Autopilot),Expected::Intent(ChoiceMode::Autopilot)),
            ("authority_guard",Interpretation::Reject(RejectReason::SemanticAuthority),Expected::Reject),
        ] {
            assert!(evaluate(id,expected,&actual).0,"{id}");
        }
    }
}
