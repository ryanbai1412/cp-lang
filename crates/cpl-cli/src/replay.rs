//! `cpl replay`, and the one call replay the debug adapter shares with it:
//! a call of a recorded run, run again with the fn as the file has it now.

use std::io::Write;

use cpl_interp::replay::Replay;
use cpl_interp::trace::{Call, Recording};

use crate::dap::{shown, Json};
use crate::{compile_side, compile_text, trace, Side, EXIT_ERRORS, EXIT_USAGE};

const USAGE: &str = "\
usage: cpl replay <file.cp> --trace <trace.json> --step <n> [--input <file>]

Runs one call of a recorded run again with the file's edited fn: the call
open at step <n> of the trace (the innermost one; --frame <k> picks the
k-th caller out), or else the call that returned into step <n>. The run before the call is the same, so the call gets the
same arguments, globals and input. Prints what the call returns and writes
now and before the edit, and the first step where it differs.

The trace is the page `cpl run --trace` writes, or its JSON (`--format
json`); it holds the source that was recorded. The input is --input, or the case `cpl stress`
saved last for the file. When the edit changes what the fn takes or returns,
or the call reads past the input, the whole file runs again and the report
says so.
";

/// The call at `step` (0-based) of a run: open there, `frame` callers out
/// of the innermost, or else the one that returned into `step`.
pub(crate) fn call_at(recorded: &Recording, step: usize, frame: usize) -> Result<Call, String> {
    // A call that changed no top-level variable has no steps of its own.
    let returned_into = || recorded.calls.iter().filter(|c| c.exit == step).min_by_key(|c| c.depth);
    recorded.call_at(step, frame).or_else(returned_into).cloned().ok_or(format!("no fn call is running at step {}", step + 1))
}

/// The call `pick` chooses of the run of `original` on `input`, replayed
/// with the fns of the file at `path`.
pub(crate) fn replay_call(
    path: &str,
    original: &str,
    input: &[u8],
    pick: &dyn Fn(&Recording) -> Result<Call, String>,
) -> Result<Replay, String> {
    let mut errors = String::new();
    let before = compile_text(path, original.to_string(), Side::Solution, &mut errors)
        .map_err(|_| format!("the recorded source does not build:\n{errors}"))?;
    let config = trace::config(original, None);
    let observed =
        cpl_interp::run_observed(&before.program, cpl_interp::Io::fixed(input), &mut Vec::new(), Some(config));
    let recorded = observed.recording.unwrap_or_default();
    let call = pick(&recorded)?;
    let mut errors = String::new();
    let after = compile_side(path, Side::Solution, &mut errors)
        .map_err(|_| format!("the edited file does not build:\n{errors}"))?;
    let replay = cpl_interp::replay::replay(
        &recorded,
        &before.program,
        &after.program,
        input,
        &call,
        trace::config(&after.text, None),
    );
    Ok(replay)
}

fn value(json: &str) -> String {
    Json::parse(json).map_or_else(|_| json.to_string(), |v| shown(&v))
}

/// The call as source: `add(i: 2)`.
pub(crate) fn call_text(name: &str, args: &str) -> String {
    let args = value(args);
    let args = args.strip_prefix('(').and_then(|a| a.strip_suffix(')')).unwrap_or(&args);
    format!("{name}({args})")
}

/// What `cpl replay` prints, and the debug adapter shows.
pub(crate) fn report(replay: &Replay) -> String {
    let call = &replay.recorded;
    let at = match call.exit > call.entry {
        true => format!("steps {}-{}", call.entry + 1, call.exit),
        false => format!("returning into step {}", call.exit + 1),
    };
    let mut out = format!("replayed call {} of `{}`, {at} of the recording\n", call.ordinal, call_text(&call.name, &call.args));
    if let Some(why) = &replay.fallback {
        out += &format!("note: {why}\n");
    }
    let shown = |json: &Option<String>| json.as_deref().map_or("nothing".into(), value);
    let (was, now) = (&replay.before.returned, &replay.after.returned);
    if replay.replayed.is_none() {
        out += "the edited run never made the call\n";
    } else if was == now {
        out += &format!("returns {} (as before)\n", shown(now));
    } else {
        out += &format!("returns {} (before the edit: {})\n", shown(now), shown(was));
    }
    let mut ids: Vec<&str> = replay.before.writes.iter().chain(&replay.after.writes).map(|(id, _)| id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    for id in ids {
        let get = |writes: &[(String, String)]| writes.iter().find(|(i, _)| i == id).map(|(_, v)| value(v));
        match (get(&replay.before.writes), get(&replay.after.writes)) {
            (b, a) if b == a => out += &format!("writes {id} = {} (as before)\n", a.unwrap_or_default()),
            (b, a) => {
                let a = a.unwrap_or_else(|| "nothing".into());
                out += &format!("writes {id} = {a} (before the edit: {})\n", b.unwrap_or_else(|| "nothing".into()));
            }
        }
    }
    if let Some(change) = &replay.earlier {
        out += &format!(
            "warning: the run already differs before the call, at step {} line {}: the edit changes an earlier call\n",
            change.step + 1,
            change.line
        );
    }
    match &replay.change {
        Some(change) => {
            let at = format!("first change: step {} line {}", change.step + 1, change.line);
            out += &match &change.variable {
                Some(var) => {
                    let shown = |json: &Option<String>| json.as_deref().map_or("nothing".into(), value);
                    format!("{at}: `{var}` = {} (before the edit: {})\n", shown(&change.after), shown(&change.before))
                }
                None => {
                    let was = change.before.as_deref().unwrap_or("the call had returned");
                    let now = change.after.as_deref().unwrap_or("the call has returned");
                    format!("{at}: {now} (before the edit: {was})\n")
                }
            };
        }
        None if replay.replayed.is_some() => out += "the call runs as recorded\n",
        None => {}
    }
    if let Some(trap) = &replay.trap {
        out += &format!("the replay stopped: {}\n", trap.message);
    }
    out
}

pub(crate) fn cli(rest: &[&str], stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    if rest.iter().any(|arg| matches!(*arg, "-h" | "--help")) {
        let _ = stdout.write_all(USAGE.as_bytes());
        return 0;
    }
    let mut usage = |message: &str| {
        *stderr += &format!("cpl: {message}\n\n{USAGE}");
        EXIT_USAGE
    };
    let (mut file, mut trace_path, mut step, mut frame, mut input) = (None, None, None, 0, None);
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        match *arg {
            "--trace" => trace_path = args.next(),
            "--step" => match args.next().and_then(|n| n.parse::<usize>().ok()).filter(|n| *n > 0) {
                Some(n) => step = Some(n - 1),
                None => return usage("--step takes a step number, from 1"),
            },
            "--frame" => match args.next().and_then(|n| n.parse().ok()) {
                Some(n) => frame = n,
                None => return usage("--frame takes a number, 0 for the innermost call"),
            },
            "--input" => input = args.next(),
            other if file.is_none() && !other.starts_with('-') => file = Some(other),
            other => return usage(&format!("unexpected `{other}`")),
        }
    }
    let (Some(file), Some(trace_path), Some(step)) = (file, trace_path, step) else {
        return usage("`replay` needs a file, --trace and --step");
    };
    let recorded = match std::fs::read_to_string(trace_path) {
        Ok(text) => text,
        Err(error) => {
            *stderr += &format!("cpl: can't read `{trace_path}`: {error}\n");
            return EXIT_USAGE;
        }
    };
    let Some(original) = Json::parse(trace::data(&recorded))
        .ok()
        .and_then(|json| json.get("source")?.get("code")?.as_str().map(str::to_string))
    else {
        *stderr += &format!("cpl: `{trace_path}` is not a trace: write one with `cpl run --trace`\n");
        return EXIT_USAGE;
    };
    let pinned = trace::pinned_input(std::path::Path::new(file));
    let named = input.map(std::path::PathBuf::from).or(pinned);
    let input = match &named {
        None => Vec::new(),
        Some(path) => match std::fs::read(path) {
            Ok(input) => input,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{}`: {error}\n", path.display());
                return EXIT_USAGE;
            }
        },
    };
    match replay_call(file, &original, &input, &|recorded| call_at(recorded, step, frame)) {
        Ok(replay) => {
            let _ = stdout.write_all(report(&replay).as_bytes());
            0
        }
        Err(message) => {
            *stderr += &format!("cpl: {message}\n");
            EXIT_ERRORS
        }
    }
}
