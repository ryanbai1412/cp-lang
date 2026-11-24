//! The exports of the `.wasm` file.
//!
//! The interface is a handful of plain functions over the module's memory,
//! with no generated bindings: `js/cpl.js` is the whole other side. A call
//! goes like this:
//!
//! 1. For each argument, `cpl_input(slot, len)` returns where to write
//!    `len` bytes: the file name, the source text, the standard input.
//! 2. One command runs on what was written: `cpl_check()`, `cpl_lint()`, `cpl_run(stack)`,
//!    `cpl_trace(stack)`, `cpl_replay(step, frame, stack)`,
//!    `cpl_hover(offset)`, ...
//! 3. `cpl_output_ptr(slot)` and `cpl_output_len(slot)` say where the
//!    answer is: JSON text, and for a run the bytes the program printed.
//!
//! The module owns every buffer, so the caller never frees anything, and a
//! pointer is good until the next call.
//!
//! The module is built with `panic = "abort"`, so a bug in the compiler
//! ends the call as a WebAssembly trap. The panic message is kept in the
//! `PANIC` output for the caller to report; after a trap the caller makes a
//! new instance.

use crate::Playground;
use std::cell::RefCell;

/// Input: the file's name, as shown in diagnostics.
pub const NAME: u32 = 0;
/// Input: the source text, or the message for `cpl_lsp`.
pub const TEXT: u32 = 1;
/// Input: the program's standard input.
pub const STDIN: u32 = 2;
/// Input: the edited source text, for `cpl_replay`.
pub const EDITED: u32 = 3;

/// Output: the command's result, as JSON.
pub const JSON: u32 = 0;
/// Output: the bytes the program printed.
pub const STDOUT: u32 = 1;
/// Output: the message of the panic that ended the last call.
pub const PANIC: u32 = 2;

#[derive(Default)]
struct State {
    playground: Playground,
    inputs: [Vec<u8>; 4],
    json: String,
    stdout: Vec<u8>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
    /// Apart from `STATE`, which a panicking call still has borrowed.
    static PANIC_MESSAGE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Runs one command on the inputs and stores its JSON result.
fn command(run: impl FnOnce(&mut Playground, &str, &str, &[u8], &mut Vec<u8>) -> String) {
    #[cfg(target_family = "wasm")]
    {
        static HOOK: std::sync::Once = std::sync::Once::new();
        HOOK.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                PANIC_MESSAGE.with(|message| *message.borrow_mut() = info.to_string());
            }));
        });
    }
    #[cfg(feature = "lazy-std")]
    cpl_check::take_std_needs();
    STATE.with(|state| {
        let state = &mut *state.borrow_mut();
        let name = String::from_utf8_lossy(&state.inputs[NAME as usize]);
        let text = String::from_utf8_lossy(&state.inputs[TEXT as usize]);
        state.stdout.clear();
        state.json = run(
            &mut state.playground,
            &name,
            &text,
            &state.inputs[STDIN as usize],
            &mut state.stdout,
        );
    });
}

/// Makes input `slot` exactly `len` bytes long and returns where to write
/// them. An unknown slot returns null.
#[no_mangle]
pub extern "C" fn cpl_input(slot: u32, len: u32) -> *mut u8 {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        match state.inputs.get_mut(slot as usize) {
            Some(input) => {
                input.clear();
                input.resize(len as usize, 0);
                input.as_mut_ptr()
            }
            None => std::ptr::null_mut(),
        }
    })
}

/// Where output `slot` starts. Good until the next call into the module.
#[no_mangle]
pub extern "C" fn cpl_output_ptr(slot: u32) -> *const u8 {
    match slot {
        JSON => STATE.with(|state| state.borrow().json.as_ptr()),
        STDOUT => STATE.with(|state| state.borrow().stdout.as_ptr()),
        PANIC => PANIC_MESSAGE.with(|message| message.borrow().as_ptr()),
        _ => std::ptr::null(),
    }
}

/// How many bytes output `slot` holds.
#[no_mangle]
pub extern "C" fn cpl_output_len(slot: u32) -> u32 {
    let len = match slot {
        JSON => STATE.with(|state| state.borrow().json.len()),
        STDOUT => STATE.with(|state| state.borrow().stdout.len()),
        PANIC => PANIC_MESSAGE.with(|message| message.borrow().len()),
        _ => 0,
    };
    len as u32
}

/// The toolchain's version, and the `stack` bytes to start `cpl_run` with.
#[no_mangle]
pub extern "C" fn cpl_version() {
    command(|_, _, _, _, _| Playground::version().to_string());
}

/// The std modules `TEXT` needs and this module lacks, as a JSON array.
/// Only in a `lazy-std` build: cpl.js reads its absence as "carries all".
#[cfg(feature = "lazy-std")]
#[no_mangle]
pub extern "C" fn cpl_std_needed() {
    command(|playground, name, text, _, _| playground.std_needed(name, text).to_string());
}

/// Gives this module `TEXT` as the std module `NAME` (`"sparse"`).
#[cfg(feature = "lazy-std")]
#[no_mangle]
pub extern "C" fn cpl_provide_std() {
    command(|playground, name, text, _, _| playground.provide_std(name, text).to_string());
}

/// Checks `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_check() {
    command(|playground, name, text, _, _| playground.check(name, text).to_string());
}

/// Lints `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_lint() {
    command(|playground, name, text, _, _| playground.lint(name, text).to_string());
}

/// `cpl fix` on `TEXT`: the fixed text and the errors left.
#[no_mangle]
pub extern "C" fn cpl_fix() {
    command(|playground, name, text, _, _| playground.fix(name, text).to_string());
}

/// Runs `TEXT` on `STDIN`, stopping it when its calls take more than
/// `stack` bytes.
/// What it printed is the `STDOUT` output.
#[no_mangle]
pub extern "C" fn cpl_run(stack: u32) {
    command(|playground, name, text, stdin, stdout| {
        let run = playground.run_with_stack(name, text, stdin, stack);
        *stdout = run.stdout;
        run.result.to_string()
    });
}

/// Replays the call open at `step` (0-based) of the run of `TEXT` on
/// `STDIN`, `frame` callers out, with the fns of `EDITED`.
#[no_mangle]
pub extern "C" fn cpl_replay(step: u32, frame: u32, stack: u32) {
    let edited = STATE.with(|state| {
        String::from_utf8_lossy(&state.borrow().inputs[EDITED as usize]).into_owned()
    });
    command(|playground, name, text, stdin, _| {
        playground.replay(name, text, &edited, stdin, step, frame, stack)
    });
}

/// Runs `TEXT` on `STDIN` like `cpl_run` and records it: the result has
/// the run's `trace` (`tools/trace/trace-schema.json`) and `debug` text.
#[no_mangle]
pub extern "C" fn cpl_trace(stack: u32) {
    command(|playground, name, text, stdin, stdout| {
        let (printed, json) = playground.trace_with_stack(name, text, stdin, stack);
        *stdout = printed;
        json
    });
}

/// Runs `cpl stress --explain --format json` on `TEXT` in-process.
#[cfg(feature = "build")]
#[no_mangle]
pub extern "C" fn cpl_stress(
    cases: u32,
    seed_lo: u32,
    seed_hi: u32,
    size: u32,
    stack: u32,
    time_ms: u32,
) {
    let seed = u64::from(seed_lo) | (u64::from(seed_hi) << 32);
    command(|playground, name, text, _, _| {
        let limits = crate::StressLimits { stack, time_ms };
        playground.stress_with_stack(name, text, cases, seed, size, limits)
    });
}

/// A random input for `TEXT` from its `read()` types.
#[cfg(feature = "build")]
#[no_mangle]
pub extern "C" fn cpl_sample(seed_lo: u32, seed_hi: u32, size: u32) {
    let seed = u64::from(seed_lo) | (u64::from(seed_hi) << 32);
    command(|playground, _, text, _, _| playground.sample(text, seed, size));
}

/// Builds `TEXT` into C++17.
#[cfg(feature = "build")]
#[no_mangle]
pub extern "C" fn cpl_build() {
    command(|playground, name, text, _, _| playground.build(name, text).to_string());
}

/// Runs `cpl explain` on `TEXT`.
#[cfg(feature = "build")]
#[no_mangle]
pub extern "C" fn cpl_explain() {
    command(|playground, name, text, _, _| playground.explain(name, text).to_string());
}

/// The hover at UTF-16 offset `offset` of `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_hover(offset: u32) {
    command(|playground, name, text, _, _| playground.hover(name, text, offset).to_string());
}

/// The declaration of the name at UTF-16 offset `offset` of `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_definition(offset: u32) {
    command(|playground, name, text, _, _| playground.definition(name, text, offset).to_string());
}

/// The outline of `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_symbols() {
    command(|playground, name, text, _, _| playground.symbols(name, text).to_string());
}

/// Handles the Language Server Protocol message in `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_lsp() {
    command(|playground, _, text, _, _| playground.lsp(text).to_string());
}
