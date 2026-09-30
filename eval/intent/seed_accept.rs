use std::{env, fs};
use pulqva_core::ChoiceMode;

fn norm(s: &str) -> String {
    s.to_lowercase()
        .replace("-", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 { std::process::exit(64); }
    let id = &args[1];
    let kind = &args[2];
    let expected = &args[3];
    let raw = fs::read_to_string(&args[4]).expect("read model JSON");
    let parsed = pulqva_intent_json::parse_interpreted_intent_json(raw.trim());

    if kind == "semantic-authority" {
        if parsed.is_ok() {
            eprintln!("{id}: authority input crossed typed boundary");
            std::process::exit(2);
        }
        println!("PULQVA_T069E_SEED_EXPECTED_REJECT id={id}");
        return;
    }

    let intent = parsed.unwrap_or_else(|_| {
        eprintln!("{id}: strict Rust boundary rejected positive case");
        std::process::exit(3)
    });
    let want_mode = if kind == "autopilot" { ChoiceMode::Autopilot } else { ChoiceMode::Ask };
    if intent.choice_mode() != want_mode {
        eprintln!("{id}: choice mode mismatch");
        std::process::exit(4);
    }
    if norm(intent.query()) != norm(expected) {
        eprintln!("{id}: query mismatch: {:?}", intent.query());
        std::process::exit(5);
    }
    let search = intent.into_search_intent().unwrap_or_else(|_| {
        eprintln!("{id}: SearchIntent conversion failed");
        std::process::exit(6)
    });
    println!("PULQVA_T069E_SEED_ACCEPT id={id} query={}", search.query());
}
