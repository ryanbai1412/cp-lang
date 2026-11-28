//! The run's stack (R13.1.2).

use super::*;

/// The stack runs out at the run's limit (contract 0's `Limits`), on the reference interpreter
/// and the engine alike.
#[test]
fn the_stack_runs_out_at_the_runs_limit() {
    use cpl_next_engine::{Io, Limits, RunInput};
    let src = "fn f(k: Int) -> Int = if k == 0: 0 else: 1 + f(k - 1)\nprint(f(100))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (p, fs) = c.form(Form::Raw);
    for (stack, out) in [(1 << 10, true), (64 << 10, false)] {
        let input = || RunInput {
            io: Io::Fixed(Vec::new()),
            limits: Limits {
                stack,
                ..Limits::default()
            },
            out: None,
        };
        let raw = cpl_next_interp::run(p, fs, input());
        let fast = cpl_next_engine::run(&c.prog, &c.opt, input());
        for o in [&raw, &fast] {
            assert_eq!(
                o.trap.as_ref().map(|t| t.code()),
                out.then_some("R0601"),
                "{stack}"
            );
            assert_eq!(o.output, if out { "" } else { "100\n" }, "{stack}");
        }
    }
}

/// A recursion past the host's stack traps R0601 in the C++ built with the sanitizers too, as it
/// does in a plain build and on the engine: the sanitizer leaves the stack's guard to the runtime.
#[test]
fn the_sanitized_cpp_runs_out_of_stack_with_r0601() {
    let src = "fn f(k: Int) -> Int = if k == 0: 0 else: 1 + f(k - 1)\nprint(f(100_000_000))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    assert_eq!(c.run("").trap.as_ref().map(|t| t.code()), Some("R0601"));
    let flags = [
        "-std=c++17",
        "-O1",
        "-g",
        "-fsanitize=address,undefined",
        "-w",
        "-pthread",
    ];
    for sanitized in [false, true] {
        let flags = [
            &flags[..2],
            if sanitized { &flags[2..4] } else { &[] },
            &flags[4..],
            &["-x", "c++"],
        ]
        .concat();
        let (bin, _) = cxx::build(c.cpp().as_bytes(), &flags).expect("g++ builds it");
        let ran = std::process::Command::new(bin).output().expect("runs");
        let err = String::from_utf8_lossy(&ran.stderr);
        assert_eq!(ran.status.code(), Some(3), "{sanitized}: {err}");
        assert!(err.contains("trap: R0601: line 1"), "{sanitized}: {err}");
    }
}
