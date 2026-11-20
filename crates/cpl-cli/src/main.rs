use std::io::{Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `cpl lsp` and `cpl lsp --stdio` are the same: stdio is the only transport.
    if args.first().map(String::as_str) == Some("lsp") && args[1..].iter().all(|a| a == "--stdio") {
        return ExitCode::from(cpl_cli::lsp());
    }
    let mut read_stdin = || {
        let mut input = Vec::new();
        let _ = std::io::stdin().lock().read_to_end(&mut input);
        input
    };
    let mut stdout = std::io::stdout();
    let mut stderr = String::new();
    let code = cpl_cli::exec(&args, &mut read_stdin, &mut stdout, &mut stderr);
    let _ = stdout.flush();
    eprint!("{stderr}");
    ExitCode::from(code)
}
