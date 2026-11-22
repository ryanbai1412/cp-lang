//! Replaying one call of a recorded run with its fn edited.
//!
//! The run is deterministic, so the edited file is run again on the same
//! input up to the call: that rebuilds the arguments, the globals and the
//! input cursor exactly, whatever their types. The run stops when the call
//! returns, and what the call did is compared with the recording.

use crate::trace::{first_change_in, Call, Change, Recording, TraceConfig};
use crate::{run_observed, Io, Trap};
use cpl_ir::{Program, Ty};

/// What a call did: its result and each top-level variable it changed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effect {
    /// JSON.
    pub returned: Option<String>,
    /// Variable ids and their JSON values when the call returned; `stdout`
    /// is what it printed.
    pub writes: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct Replay {
    /// The call as recorded, and as the edited fn ran it.
    pub recorded: Call,
    pub replayed: Option<Call>,
    pub before: Effect,
    pub after: Effect,
    /// The first step of the replayed call that differs from the recording,
    /// as a step of [`Replay::recording`]; `variable` is `returned` when
    /// only the result differs.
    pub change: Option<Change>,
    /// A step before the call that already differs: the edit changes an
    /// earlier call of the fn too.
    pub earlier: Option<Change>,
    /// Why the whole file ran instead of the one call.
    pub fallback: Option<String>,
    pub recording: Recording,
    pub trap: Option<Trap>,
}

/// Runs `call` of `recorded`, a run of `original` on `input`, again with the
/// fns of `edited`.
pub fn replay(
    recorded: &Recording,
    original: &Program,
    edited: &Program,
    input: &[u8],
    call: &Call,
    config: TraceConfig,
) -> Replay {
    let changed = signature(original, &call.name) != signature(edited, &call.name);
    let fallback = changed.then(|| match signature(edited, &call.name) {
        None => format!("the edit removes `{}`, so the whole file ran again", call.name),
        Some(_) => format!("the edit changes what `{}` takes or returns, so the whole file ran again", call.name),
    });
    let stop = fallback.is_none().then_some((call.name.clone(), call.ordinal));
    let mut run = run(edited, input, TraceConfig { stop_after_call: stop.clone(), ..config.clone() });
    let mut fallback = fallback;
    let mut replayed = find(&run.0, call);
    let past_input = run.1.as_ref().is_some_and(|trap| trap.message.contains("the input ended"));
    if stop.is_some() && (replayed.is_none() || past_input) {
        fallback = Some(match past_input {
            true => format!("the edited `{}` reads past the recorded input, so the whole file ran again", call.name),
            false => format!("the edited file never makes call {} of `{}`, so the whole file ran again", call.ordinal, call.name),
        });
        run = self::run(edited, input, TraceConfig { stop_after_call: None, ..config });
        replayed = find(&run.0, call);
    }
    let (recording, trap) = run;
    let earlier = first_change_in(recorded, 0..call.entry, &recording, 0..replayed.as_ref().map_or(recording.steps.len(), |c| c.entry));
    let change = match &replayed {
        None => None,
        Some(new) => first_change_in(recorded, call.entry..call.exit, &recording, new.entry..new.exit).or_else(|| {
            // A call with no steps of its own shows its result in the step
            // it returns into, which the replay stops before: the recorded
            // run is the same up to there.
            let step = if new.exit > new.entry { new.exit - 1 } else { new.exit };
            let line = recording.steps.get(step).or(recorded.steps.get(call.exit)).map_or(0, |s| s.line);
            (new.returned != call.returned).then(|| Change {
                step,
                line,
                variable: Some("returned".into()),
                before: call.returned.clone(),
                after: new.returned.clone(),
            })
        }),
    };
    Replay {
        before: effect(recorded, call),
        after: replayed.as_ref().map(|new| effect(&recording, new)).unwrap_or_default(),
        recorded: call.clone(),
        replayed,
        change,
        earlier,
        fallback,
        recording,
        trap,
    }
}

fn run(program: &Program, input: &[u8], config: TraceConfig) -> (Recording, Option<Trap>) {
    let observed = run_observed(program, Io::fixed(input), &mut Vec::new(), Some(config));
    (observed.recording.unwrap_or_default(), observed.trap)
}

fn find(recording: &Recording, call: &Call) -> Option<Call> {
    recording.calls.iter().find(|c| c.name == call.name && c.ordinal == call.ordinal).cloned()
}

/// What a fn takes (by value or by `mut`, and the types) and returns.
fn signature(program: &Program, name: &str) -> Option<(Vec<(bool, Ty)>, Ty)> {
    let f = program.fns.iter().find(|f| f.name == name)?;
    let params = f.params.iter().zip(&f.locals).map(|(p, l)| (p.mutable, l.ty.clone())).collect();
    Some((params, f.ret.clone()))
}

fn effect(recording: &Recording, call: &Call) -> Effect {
    let mut writes = Vec::new();
    if call.exit > call.entry {
        let end = recording.values_at(call.exit - 1);
        let start = call.entry.checked_sub(1).map(|i| recording.values_at(i)).unwrap_or_default();
        for (id, value) in end {
            if start.get(id) != Some(&value) {
                writes.push((id.to_string(), value.to_string()));
            }
        }
    }
    Effect { returned: call.returned.clone(), writes }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUN: &str = "\
var total = 0
fn add(i: Int) -> Int:
    total += i
    return total
for i in 1..=3:
    print(add(i))
";

    fn program(source: &str) -> Program {
        let mut db = cpl_check::Database::new();
        db.set_file("t.cp", source);
        let analysis = db.analyze("t.cp").expect("the file was just set");
        assert!(analysis.diagnostics.is_empty(), "{:?}", analysis.diagnostics);
        cpl_ir::lower(&analysis).expect("lowers")
    }

    fn config(source: &str) -> TraceConfig {
        TraceConfig { source: source.into(), record_frames: true, ..TraceConfig::default() }
    }

    fn replayed(edit: &str, input: &str, ordinal: usize) -> Replay {
        let original = program(RUN);
        let (recorded, trap) = run(&original, input.as_bytes(), config(RUN));
        assert!(trap.is_none());
        let call = recorded.calls.iter().find(|c| c.name == "add" && c.ordinal == ordinal).unwrap().clone();
        replay(&recorded, &original, &program(edit), input.as_bytes(), &call, config(edit))
    }

    #[test]
    fn the_calls_of_a_run_are_recorded_with_their_arguments_and_results() {
        let (recorded, _) = run(&program(RUN), b"", config(RUN));
        let calls: Vec<_> = recorded.calls.iter().map(|c| (c.ordinal, c.args.as_str(), c.returned.as_deref())).collect();
        assert_eq!(calls, [(1, "{\"i\":1}", Some("1")), (2, "{\"i\":2}", Some("3")), (3, "{\"i\":3}", Some("6"))]);
        let second = &recorded.calls[1];
        assert_eq!(recorded.call_at(second.entry, 0), Some(second));
        assert_eq!(recorded.steps[second.entry].line, 3);
    }

    #[test]
    fn an_edited_call_runs_to_its_return_and_names_its_first_change() {
        let edit = RUN.replace("    total += i\n", "    total += i\n    if i == 2: total += 10\n");
        let replay = replayed(&edit, "", 2);
        assert_eq!(replay.fallback, None);
        assert_eq!(replay.earlier, None, "call 1 behaves as before");
        assert_eq!(replay.before.returned.as_deref(), Some("3"));
        assert_eq!(replay.after.returned.as_deref(), Some("13"));
        assert!(replay.after.writes.contains(&("total".into(), "13".into())), "{:?}", replay.after.writes);
        let change = replay.change.expect("the call differs");
        assert_eq!(change.line, 4, "the new `if` is where the call first departs");
        assert!(replay.recording.calls.iter().all(|c| c.ordinal <= 2), "the run stops when the call returns");
        assert!(replay.trap.is_none());
    }

    #[test]
    fn an_unchanged_call_replays_the_same() {
        let replay = replayed(RUN, "", 3);
        assert_eq!((replay.change, replay.earlier, replay.fallback), (None, None, None));
        assert_eq!(replay.before, replay.after);
    }

    #[test]
    fn a_changed_signature_or_reading_past_the_input_runs_the_whole_file() {
        let wider = RUN.replace("fn add(i: Int) -> Int:", "fn add(i: Int) -> Bool:\n    total += i\n    return total > 0\nfn old(i: Int) -> Int:");
        let replay = replayed(&wider, "", 2);
        assert!(replay.fallback.as_deref().is_some_and(|why| why.contains("takes or returns")), "{:?}", replay.fallback);
        let reads = RUN.replace("    total += i\n", "    let more: Int = read()\n    total += i + more\n");
        let replay = replayed(&reads, "5\n", 2);
        assert!(replay.fallback.as_deref().is_some_and(|why| why.contains("past the recorded input")), "{:?}", replay.fallback);
        assert!(replay.earlier.is_some(), "call 1 already read the 5");
    }
}
