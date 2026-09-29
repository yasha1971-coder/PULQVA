use std::{env, io::{self, Read}, thread, time::Duration};

fn main() {
    let mode = env::args().nth(1).unwrap_or_default();
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    match mode.as_str() {
        "valid" => print!(r#"{"query":"countdown video","choice_mode":"ask"}"#),
        "malformed" => print!(r#"{"query":"broken""#),
        "oversized" => print!("{}", "x".repeat(5000)),
        "crash" => std::process::exit(17),
        "timeout" => thread::sleep(Duration::from_secs(10)),
        _ => std::process::exit(18),
    }
}
