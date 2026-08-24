//! `weavatrix-seo` binary.

use std::io::{self, Write};
use std::process::ExitCode;
use weavatrix_seo_mcp::{HostOptions, parse_host_args, serve};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|item| item == "mcp") {
        return run_mcp(&args[1..]);
    }
    let output = weavatrix_seo_cli::run(&args);
    let _ = io::stdout().write_all(output.stdout.as_bytes());
    let _ = io::stderr().write_all(output.stderr.as_bytes());
    ExitCode::from(u8::try_from(output.code).unwrap_or(1))
}

fn run_mcp(args: &[String]) -> ExitCode {
    if args.iter().any(|item| item == "--help" || item == "-h") {
        println!(
            "weavatrix-seo mcp — Weavatrix SEO MCP\n\nUsage:\n  weavatrix-seo mcp [--max-pages N]"
        );
        return ExitCode::SUCCESS;
    }
    match parse_host_args(args).and_then(|options: HostOptions| serve(&options)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("weavatrix-seo mcp: {error}");
            ExitCode::FAILURE
        }
    }
}
