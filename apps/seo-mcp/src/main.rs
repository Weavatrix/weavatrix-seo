//! `weavatrix-seo-mcp` stdio host.

use std::env;
use std::process::ExitCode;
use weavatrix_seo_mcp::{parse_host_args, serve};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "weavatrix-seo-mcp — Weavatrix SEO MCP\n\nUsage:\n  weavatrix-seo-mcp [--max-pages N]"
        );
        return ExitCode::SUCCESS;
    }
    match parse_host_args(&args).and_then(|options| serve(&options)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("weavatrix-seo-mcp: {error}");
            ExitCode::FAILURE
        }
    }
}
