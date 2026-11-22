use std::io::{Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    cpl_cli::install_ice_hook();
    let args: Vec<String> = std::env::args().skip(1).collect();
    cpl_cli::set_crash_context(&args);
    if let Some(code) = cpl_cli::toolchain::launch(&args) {
        return ExitCode::from(code);
    }
    if std::env::var_os("CPL_FORCE_ICE").as_deref() == Some(std::ffi::OsStr::new("1")) {
        panic!("forced ICE for testing");
    }
    // `cpl lsp` and `cpl lsp --stdio` are the same: stdio is the only transport.
    // The editor checks against edition 0.3 unless `--edition=0.2` keeps today's.
    let lsp_flag = |a: &String| matches!(a.as_str(), "--stdio" | "--edition=0.2" | "--edition=0.3");
    if args.first().map(String::as_str) == Some("lsp") && args[1..].iter().all(lsp_flag) {
        if !args.iter().any(|a| a == "--edition=0.2") {
            cpl_cli::default_to_spec_edition();
        }
        return ExitCode::from(cpl_cli::lsp());
    }
    if args.first().map(String::as_str) == Some("dap") && args[1..].iter().all(|a| a == "--stdio") {
        return ExitCode::from(cpl_cli::dap());
    }
    // What has arrived on standard input, without waiting for its end;
    // nothing once it has ended.
    let mut read_stdin = || {
        let mut chunk = vec![0; 1 << 16];
        let read = std::io::stdin().lock().read(&mut chunk).unwrap_or(0);
        chunk.truncate(read);
        chunk
    };
    let mut stdout = std::io::stdout();
    let mut stderr = String::new();
    let code = cpl_cli::exec(&args, &mut read_stdin, &mut stdout, &mut stderr);
    let _ = stdout.flush();
    eprint!("{stderr}");
    ExitCode::from(code)
}
