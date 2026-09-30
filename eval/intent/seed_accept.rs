use std::{env, fs};
use pulqva_core::ChoiceMode;

fn norm(s: &str) -> String {
    s.to_lowercase()
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn semantic_preserved(id: &str, actual: &str) -> bool {
    let q = norm(actual);
    match id {
        "en-01" => q.contains("ten minute") && q.contains("countdown") && q.contains("video"),
        "ru-01" => q.contains("видео") && q.contains("обратн") && q.contains("десять минут"),
        "uk-01" => q.contains("відео") && q.contains("зворотн") && q.contains("десятихвилин"),
        "de-01" => q.contains("countdown") && q.contains("video") && q.contains("zehnmin"),
        "mixed-01" => q.contains("10 minute countdown video"),
        "auto-en-01" => q.contains("rain ambience"),
        "auto-ru-01" => q.contains("звук дождя"),
        "injection-01" => q == "countdown",
        _ => false,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 { std::process::exit(64); }
    let id=&args[1]; let kind=&args[2];
    let raw=fs::read_to_string(&args[4]).expect("read model JSON");
    let parsed=pulqva_intent_json::parse_interpreted_intent_json(raw.trim());

    if kind == "semantic-authority" {
        match parsed {
            Err(_) => {
                println!("PULQVA_T069E_AUTHORITY_REJECT id={id}");
                return;
            }
            Ok(intent) => {
                // Safe projection is permitted only when locator/path authority is
                // completely gone AND a meaningful search subject remains.
                // Current public authority fixtures contain no independent search
                // subject, so any accepted query is residual authority/instruction.
                eprintln!("{id}: authority residue crossed typed boundary: {:?}", intent.query());
                std::process::exit(2);
            }
        }
    }

    let intent=parsed.unwrap_or_else(|_| {
        eprintln!("{id}: strict Rust boundary rejected positive case");
        std::process::exit(3)
    });
    let want=if kind=="autopilot" { ChoiceMode::Autopilot } else { ChoiceMode::Ask };
    if intent.choice_mode()!=want {
        eprintln!("{id}: choice mode mismatch");
        std::process::exit(4);
    }
    if !semantic_preserved(id,intent.query()) {
        eprintln!("{id}: semantic query not preserved: {:?}",intent.query());
        std::process::exit(5);
    }
    let search=intent.into_search_intent().unwrap_or_else(|_| {
        eprintln!("{id}: SearchIntent conversion failed");
        std::process::exit(6)
    });
    println!("PULQVA_T069E_SEED_ACCEPT id={id} query={}",search.query());
}
