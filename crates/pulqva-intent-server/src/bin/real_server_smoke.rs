use pulqva_intent_http::{build_interpretation_request,interpret_via_loopback,LoopbackIntentEndpoint};
use pulqva_intent_server::{IntentServer,IntentServerPlan,ReadinessDiagnostics};
use std::{env,error::Error,fs,path::PathBuf,process::ExitCode,time::Duration};

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
    let mut receipt=serde_json::json!({"protocol":"t069f-real-server-v1","scope":"transport smoke, not semantic model acceptance",
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
    let cases=[("en","find a countdown video"),("ru","найди видео обратного отсчёта"),("uk","знайди відео зворотного відліку")];
    let mut rows=Vec::new();
    for (id,input) in cases {
        let outcome=build_interpretation_request(input,&schema).and_then(|request|interpret_via_loopback(endpoint,&request));
        match outcome {
            Ok(x)=>rows.push(serde_json::json!({"id":id,"ok":true,"interpretation":format!("{x:?}")})),
            Err(e)=>rows.push(serde_json::json!({"id":id,"ok":false,"error":format!("{e:?}")})),
        }
    }
    let ok=rows.iter().all(|x|x["ok"]==true);
    // Do not call process::exit while the server owner is alive: it skips Drop.
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
