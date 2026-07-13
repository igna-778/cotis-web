use std::path::PathBuf;

use cotis_web_builder::{WebCotisOptions, execute_web_build};

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut opts = WebCotisOptions::default();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--output" => {
                if let Some(v) = it.next() {
                    opts.output = PathBuf::from(v);
                }
            }
            "--features" => {
                if let Some(v) = it.next() {
                    for feature in v.split(',') {
                        let trimmed = feature.trim().trim_matches('"').trim_matches('\'');
                        if trimmed.is_empty() {
                            continue;
                        }
                        if !opts.features.iter().any(|existing| existing == trimmed) {
                            opts.features.push(trimmed.to_string());
                        }
                    }
                }
            }
            "-h" | "--help" => {
                eprintln!(
                    "Usage: cotis-web-builder-cli [--output <path>] [--features <f1,f2,...>]"
                );
                return;
            }
            _ => {}
        }
    }

    if let Err(e) = execute_web_build(opts) {
        eprintln!("cotis-web-builder failed: {:?}", e);
        std::process::exit(1);
    }
}
