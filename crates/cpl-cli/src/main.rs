use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `cpl lsp` and `cpl lsp --stdio` are the same: stdio is the only transport.
    if args.first().map(String::as_str) == Some("lsp") && args[1..].iter().all(|a| a == "--stdio") {
        return ExitCode::from(cpl_cli::lsp());
    }
    let out = cpl_cli::run(&args);
    print!("{}", out.stdout);
    eprint!("{}", out.stderr);
    ExitCode::from(out.code)
}
