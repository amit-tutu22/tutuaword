//! Headless CLI for the document automation API (F26.S4).

use std::env;
use std::fs;
use tw_automation::{envelope_from_json_str, AutomationSession};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: tw-automation-cli <request.json>\n\
             Example: {{\"schema_version\":1,\"type\":\"new_document\"}}"
        );
        std::process::exit(2);
    }

    let json = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", args[1]);
        std::process::exit(2);
    });

    let envelope = envelope_from_json_str(&json).unwrap_or_else(|e| {
        eprintln!("Invalid request JSON: {e:?}");
        std::process::exit(2);
    });

    let mut session = AutomationSession::new();
    let response = session.execute(envelope);
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
    if !response.success {
        std::process::exit(1);
    }
}
