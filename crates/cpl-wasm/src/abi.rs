//! The exports of the `.wasm` file.
//!
//! The interface is a handful of plain functions over the module's memory,
//! with no generated bindings: `js/cpl.js` is the whole other side. A call
//! goes like this:
//!
//! 1. For each argument, `cpl_input(slot, len)` returns where to write
//!    `len` bytes: the file name, the source text, the standard input.
//! 2. One command runs on what was written: `cpl_check()`, `cpl_run(depth)`,
//!    `cpl_trace(depth)`,
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

/// Output: the command's result, as JSON.
pub const JSON: u32 = 0;
/// Output: the bytes the program printed.
pub const STDOUT: u32 = 1;
/// Output: the message of the panic that ended the last call.
pub const PANIC: u32 = 2;

#[derive(Default)]
struct State {
    playground: Playground,
    inputs: [Vec<u8>; 3],
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

/// The toolchain's version, and the `depth` to start `cpl_run` with.
#[no_mangle]
pub extern "C" fn cpl_version() {
    command(|_, _, _, _, _| Playground::version().to_string());
}

/// Checks `TEXT`.
#[no_mangle]
pub extern "C" fn cpl_check() {
    command(|playground, name, text, _, _| playground.check(name, text).to_string());
}

/// Runs `TEXT` on `STDIN`, stopping it when its calls nest `depth` deep.
/// What it printed is the `STDOUT` output.
#[no_mangle]
pub extern "C" fn cpl_run(depth: u32) {
    command(|playground, name, text, stdin, stdout| {
        let run = playground.run_to_depth(name, text, stdin, depth);
        *stdout = run.stdout;
        run.result.to_string()
    });
}

/// Runs `TEXT` on `STDIN` like `cpl_run` and records it: the result has
/// the run's `trace` (`tools/trace/trace-schema.json`) and `debug` text.
#[no_mangle]
pub extern "C" fn cpl_trace(depth: u32) {
    command(|playground, name, text, stdin, stdout| {
        let (printed, json) = playground.trace_to_depth(name, text, stdin, depth);
        *stdout = printed;
        json
    });
}

/// Builds `TEXT` into C++17.
#[no_mangle]
pub extern "C" fn cpl_build() {
    command(|playground, name, text, _, _| playground.build(name, text).to_string());
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
