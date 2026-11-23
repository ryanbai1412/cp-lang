//! Replaying one call of a recorded run with its fn edited.
//!
//! The run is deterministic, so the edited file is run again on the same
//! input up to the call: that rebuilds the arguments, the globals and the
//! input cursor exactly, whatever their types. The run stops when the call
//! returns, and what the call did is compared with the recording.

use crate::trace::{first_change_in, string, Call, Change, Recording, TraceConfig};
use crate::{observe_on_this_thread, run_observed, Io, Limits, Observed, Stop, Trap};
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

/// The call at `step` (0-based) of a run: open there, `frame` callers out
/// of the innermost, or else the one that returned into `step`.
pub fn call_at(recorded: &Recording, step: usize, frame: usize) -> Result<Call, String> {
    // A call that changed no top-level variable has no steps of its own.
    let returned_into = || {
        recorded
            .calls
            .iter()
            .filter(|c| c.exit == step)
            .min_by_key(|c| c.depth)
    };
    recorded
        .call_at(step, frame)
        .or_else(returned_into)
        .cloned()
        .ok_or(format!("no fn call is running at step {}", step + 1))
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
    replay_by(
        recorded,
        original,
        edited,
        input,
        call,
        config,
        &|program, input, config| {
            run_observed(program, Io::fixed(input), &mut Vec::new(), Some(config))
        },
    )
}

/// [`replay`] where no thread can be started (WebAssembly), its calls held
/// to `limits` and each run ended by `stop`.
pub fn replay_on_this_thread(
    recorded: &Recording,
    original: &Program,
    edited: &Program,
    input: &[u8],
    call: &Call,
    config: TraceConfig,
    limits: Limits,
    stop: Stop,
) -> Replay {
    replay_by(
        recorded,
        original,
        edited,
        input,
        call,
        config,
        &|program, input, config| {
            let io = Io::fixed(input).until(stop.clone());
            observe_on_this_thread(program, io, &mut Vec::new(), limits, Some(config))
        },
    )
}

type Runner<'a> = dyn Fn(&Program, &[u8], TraceConfig) -> Observed + 'a;

fn replay_by(
    recorded: &Recording,
    original: &Program,
    edited: &Program,
    input: &[u8],
    call: &Call,
    config: TraceConfig,
    observe: &Runner,
) -> Replay {
    let record = |program: &Program, input: &[u8], config: TraceConfig| {
        let observed = observe(program, input, config);
        (observed.recording.unwrap_or_default(), observed.trap)
    };
    let changed = signature(original, &call.name) != signature(edited, &call.name);
    let fallback = changed.then(|| match signature(edited, &call.name) {
        None => format!(
            "the edit removes `{}`, so the whole file ran again",
            call.name
        ),
        Some(_) => format!(
            "the edit changes what `{}` takes or returns, so the whole file ran again",
            call.name
        ),
    });
    let stop = fallback
        .is_none()
        .then_some((call.name.clone(), call.ordinal));
    let mut run = record(
        edited,
        input,
        TraceConfig {
            stop_after_call: stop.clone(),
            ..config.clone()
        },
    );
    let mut fallback = fallback;
    let mut replayed = find(&run.0, call);
    let past_input = run
        .1
        .as_ref()
        .is_some_and(|trap| trap.message.contains("the input ended"));
    if stop.is_some() && (replayed.is_none() || past_input) {
        fallback = Some(match past_input {
            true => format!(
                "the edited `{}` reads past the recorded input, so the whole file ran again",
                call.name
            ),
            false => format!(
                "the edited file never makes call {} of `{}`, so the whole file ran again",
                call.ordinal, call.name
            ),
        });
        run = record(
            edited,
            input,
            TraceConfig {
                stop_after_call: None,
                ..config
            },
        );
        replayed = find(&run.0, call);
    }
    let (recording, trap) = run;
    let earlier = first_change_in(
        recorded,
        0..call.entry,
        &recording,
        0..replayed.as_ref().map_or(recording.len(), |c| c.entry),
    );
    let change = match &replayed {
        None => None,
        Some(new) => first_change_in(
            recorded,
            call.entry..call.exit,
            &recording,
            new.entry..new.exit,
        )
        .or_else(|| {
            // A call with no steps of its own shows its result in the step
            // it returns into, which the replay stops before: the recorded
            // run is the same up to there.
            let step = if new.exit > new.entry {
                new.exit - 1
            } else {
                new.exit
            };
            let line = recording
                .line(step)
                .or(recorded.line(call.exit))
                .unwrap_or(0);
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
        after: replayed
            .as_ref()
            .map(|new| effect(&recording, new))
            .unwrap_or_default(),
        recorded: call.clone(),
        replayed,
        change,
        earlier,
        fallback,
        recording,
        trap,
    }
}

impl Replay {
    /// The report as JSON, for the trace viewer: values are the trace's
    /// JSON, `null` when there is none.
    pub fn json(&self) -> String {
        let raw =
            |out: &mut String, json: &Option<String>| *out += json.as_deref().unwrap_or("null");
        let text = |out: &mut String, text: Option<&str>| match text {
            Some(text) => string(out, text),
            None => *out += "null",
        };
        let call = &self.recorded;
        let mut out = String::from("{\"call\":{\"name\":");
        string(&mut out, &call.name);
        out += &format!(
            ",\"ordinal\":{},\"entry\":{},\"exit\":{},\"args\":{}}}",
            call.ordinal, call.entry, call.exit, call.args
        );
        out += ",\"fallback\":";
        text(&mut out, self.fallback.as_deref());
        out += &format!(
            ",\"replayed\":{},\"returned\":{{\"before\":",
            self.replayed.is_some()
        );
        raw(&mut out, &self.before.returned);
        out += ",\"after\":";
        raw(&mut out, &self.after.returned);
        out += "},\"writes\":[";
        let mut ids: Vec<&str> = self
            .before
            .writes
            .iter()
            .chain(&self.after.writes)
            .map(|(id, _)| id.as_str())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        for (i, id) in ids.into_iter().enumerate() {
            let get = |writes: &[(String, String)]| {
                writes.iter().find(|(w, _)| w == id).map(|(_, v)| v.clone())
            };
            out += if i == 0 { "{\"id\":" } else { ",{\"id\":" };
            string(&mut out, id);
            out += ",\"before\":";
            raw(&mut out, &get(&self.before.writes));
            out += ",\"after\":";
            raw(&mut out, &get(&self.after.writes));
            out += "}";
        }
        out += "],\"change\":";
        match &self.change {
            None => out += "null",
            Some(change) => {
                out += &format!(
                    "{{\"step\":{},\"line\":{},\"variable\":",
                    change.step, change.line
                );
                text(&mut out, change.variable.as_deref());
                // With no variable, `before` and `after` are step labels, not values.
                for (key, value) in [("before", &change.before), ("after", &change.after)] {
                    out += &format!(",\"{key}\":");
                    match change.variable {
                        Some(_) => raw(&mut out, value),
                        None => text(&mut out, value.as_deref()),
                    }
                }
                out += "}";
            }
        }
        out += ",\"earlier\":";
        match &self.earlier {
            None => out += "null",
            Some(change) => {
                out += &format!("{{\"step\":{},\"line\":{}}}", change.step, change.line)
            }
        }
        out += ",\"trap\":";
        text(
            &mut out,
            self.trap.as_ref().map(|trap| trap.message.as_str()),
        );
        out + "}"
    }
}

#[cfg(test)]
fn run(program: &Program, input: &[u8], config: TraceConfig) -> (Recording, Option<Trap>) {
    let observed = run_observed(program, Io::fixed(input), &mut Vec::new(), Some(config));
    (observed.recording.unwrap_or_default(), observed.trap)
}

fn find(recording: &Recording, call: &Call) -> Option<Call> {
    recording
        .calls
        .iter()
        .find(|c| c.name == call.name && c.ordinal == call.ordinal)
        .cloned()
}

/// What a fn takes (by value or by `mut`, and the types) and returns.
fn signature(program: &Program, name: &str) -> Option<(Vec<(bool, Ty)>, Ty)> {
    let f = program.fns.iter().find(|f| f.name == name)?;
    let params = f
        .params
        .iter()
        .zip(&f.locals)
        .map(|(p, l)| (p.mutable, l.ty.clone()))
        .collect();
    Some((params, f.ret.clone()))
}

fn effect(recording: &Recording, call: &Call) -> Effect {
    let mut writes = Vec::new();
    if call.exit > call.entry {
        let end = recording.values_at(call.exit - 1);
        let start = call
            .entry
            .checked_sub(1)
            .map(|i| recording.values_at(i))
            .unwrap_or_default();
        for (id, value) in end {
            if start.get(&id) != Some(&value) {
                writes.push((id, value));
            }
        }
    }
    Effect {
        returned: call.returned.clone(),
        writes,
    }
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
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        cpl_ir::lower(&analysis).expect("lowers")
    }

    fn config(source: &str) -> TraceConfig {
        TraceConfig {
            source: source.into(),
            record_frames: true,
            ..TraceConfig::default()
        }
    }

    fn replayed(edit: &str, input: &str, ordinal: usize) -> Replay {
        let original = program(RUN);
        let (recorded, trap) = run(&original, input.as_bytes(), config(RUN));
        assert!(trap.is_none());
        let call = recorded
            .calls
            .iter()
            .find(|c| c.name == "add" && c.ordinal == ordinal)
            .unwrap()
            .clone();
        replay(
            &recorded,
            &original,
            &program(edit),
            input.as_bytes(),
            &call,
            config(edit),
        )
    }

    #[test]
    fn the_calls_of_a_run_are_recorded_with_their_arguments_and_results() {
        let (recorded, _) = run(&program(RUN), b"", config(RUN));
        let calls: Vec<_> = recorded
            .calls
            .iter()
            .map(|c| (c.ordinal, c.args.as_str(), c.returned.as_deref()))
            .collect();
        assert_eq!(
            calls,
            [
                (1, "{\"i\":1}", Some("1")),
                (2, "{\"i\":2}", Some("3")),
                (3, "{\"i\":3}", Some("6"))
            ]
        );
        let second = &recorded.calls[1];
        assert_eq!(recorded.call_at(second.entry, 0), Some(second));
        assert_eq!(recorded.line(second.entry).unwrap(), 3);
    }

    #[test]
    fn an_edited_call_runs_to_its_return_and_names_its_first_change() {
        let edit = RUN.replace(
            "    total += i\n",
            "    total += i\n    if i == 2: total += 10\n",
        );
        let replay = replayed(&edit, "", 2);
        assert_eq!(replay.fallback, None);
        assert_eq!(replay.earlier, None, "call 1 behaves as before");
        assert_eq!(replay.before.returned.as_deref(), Some("3"));
        assert_eq!(replay.after.returned.as_deref(), Some("13"));
        assert!(
            replay.after.writes.contains(&("total".into(), "13".into())),
            "{:?}",
            replay.after.writes
        );
        let change = replay.change.expect("the call differs");
        assert_eq!(
            change.line, 4,
            "the new `if` is where the call first departs"
        );
        assert!(
            replay.recording.calls.iter().all(|c| c.ordinal <= 2),
            "the run stops when the call returns"
        );
        assert!(replay.trap.is_none());
    }

    #[test]
    fn an_unchanged_call_replays_the_same() {
        let replay = replayed(RUN, "", 3);
        assert_eq!(
            (replay.change, replay.earlier, replay.fallback),
            (None, None, None)
        );
        assert_eq!(replay.before, replay.after);
    }

    #[test]
    fn a_changed_signature_or_reading_past_the_input_runs_the_whole_file() {
        let wider = RUN.replace("fn add(i: Int) -> Int:", "fn add(i: Int) -> Bool:\n    total += i\n    return total > 0\nfn old(i: Int) -> Int:").replace("print(add(i))", "print(yes(add(i)))");
        let replay = replayed(&wider, "", 2);
        assert!(
            replay
                .fallback
                .as_deref()
                .is_some_and(|why| why.contains("takes or returns")),
            "{:?}",
            replay.fallback
        );
        let reads = RUN.replace(
            "    total += i\n",
            "    let more: Int = read()\n    total += i + more\n",
        );
        let replay = replayed(&reads, "5\n", 2);
        assert!(
            replay
                .fallback
                .as_deref()
                .is_some_and(|why| why.contains("past the recorded input")),
            "{:?}",
            replay.fallback
        );
        assert!(replay.earlier.is_some(), "call 1 already read the 5");
    }

    #[test]
    fn the_program_ends_at_its_last_statement_not_at_a_later_fn() {
        let source = "var x = 1\nprint(x)\n\nfn unused() -> Int:\n    return 2\n";
        let (recorded, _) = run(&program(source), b"", config(source));
        let end = recorded.step(recorded.len() - 1).unwrap();
        assert_eq!((end.line, end.label.as_str()), (2, "the program ended"));
    }

    #[test]
    fn a_fns_locals_each_take_a_step_and_a_changed_local_is_the_first_change() {
        let source = "fn sum(n: Int) -> Int:\n    var s = 0\n    for i in 1..=n:\n        s += i\n    return s\nprint(sum(2))\n";
        let (recorded, _) = run(&program(source), b"", config(source));
        let lines: Vec<u32> = (0..recorded.len())
            .filter_map(|i| recorded.line(i))
            .collect();
        assert_eq!(lines, [2, 3, 4, 3, 4, 6, 6]);
        let locals = recorded.step(4).unwrap().locals;
        assert!(locals.contains("\"s\":3"), "{locals}");
        let edit = source.replace("s += i", "s += 2 * i");
        let (after, _) = run(&program(&edit), b"", config(&edit));
        let change = crate::trace::first_change(&recorded, &after).unwrap();
        assert_eq!(
            (change.step, change.line, change.variable.as_deref()),
            (2, 4, Some("s"))
        );
    }

    #[test]
    fn the_report_as_json_has_the_call_its_results_and_the_first_change() {
        let edit = RUN.replace(
            "    total += i\n",
            "    total += i\n    if i == 2: total += 10\n",
        );
        let json = replayed(&edit, "", 2).json();
        assert!(
            json.starts_with("{\"call\":{\"name\":\"add\",\"ordinal\":2,"),
            "{json}"
        );
        assert!(
            json.contains("\"args\":{\"i\":2}},\"fallback\":null,\"replayed\":true"),
            "{json}"
        );
        assert!(
            json.contains("\"returned\":{\"before\":3,\"after\":13}"),
            "{json}"
        );
        assert!(
            json.contains("{\"id\":\"total\",\"before\":3,\"after\":13}"),
            "{json}"
        );
        assert!(
            json.contains("\"change\":{\"step\":") && json.contains("\"line\":4,"),
            "{json}"
        );
        assert!(json.ends_with("\"earlier\":null,\"trap\":null}"), "{json}");
    }

    #[test]
    fn a_replay_on_this_thread_matches_one_on_its_own_thread() {
        let edit = RUN.replace("return total", "return total * 2");
        let original = program(RUN);
        let (recorded, _) = run(&original, b"", config(RUN));
        let call = call_at(&recorded, recorded.calls[1].entry, 0).unwrap();
        let limits = Limits {
            stack_bytes: 1 << 20,
        };
        let here = replay_on_this_thread(
            &recorded,
            &original,
            &program(&edit),
            b"",
            &call,
            config(&edit),
            limits,
            Stop::new(),
        );
        let there = replay(
            &recorded,
            &original,
            &program(&edit),
            b"",
            &call,
            config(&edit),
        );
        assert_eq!(here.json(), there.json());
        assert!(here.json().contains("\"after\":6}"), "{}", here.json());
    }
}
