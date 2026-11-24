//! The debug run that explains a failing case: one for `cpl stress` and
//! the browser's in-process stress.

use crate::cli::Stage;
use crate::judge::Explaining;
use cpl_interp::trace::{
    answered_divergence, explain_divergence, stages, stuck_divergence, Divergence, Recording,
    TraceConfig,
};
use std::time::Duration;

/// How long the traced run that explains a counterexample may take.
const TRACE_LIMIT: Duration = Duration::from_secs(10);

/// The first steps of a run that never finishes: enough to see it go round.
pub const STUCK_STEPS: usize = 100_000;

/// The programs a failing case is traced through.
#[derive(Clone, Copy)]
pub struct Programs<'a> {
    pub fast: &'a cpl_ir::Program,
    /// The file's own `@alt(program)` fn, or the file with its `@alt(f)`
    /// fns swapped in.
    pub brute: Option<&'a cpl_ir::Program>,
    pub interactor: Option<&'a cpl_ir::Program>,
}

/// A failing case, traced.
pub struct Retraced {
    pub fast: Recording,
    /// The brute's run, when the two are lined up.
    pub slow: Option<Recording>,
    pub stages: Vec<Stage>,
    /// What explains the case. A part's wrong call is always found: calls
    /// are always kept. The rest needs `explain`.
    pub divergence: Result<Option<Divergence>, String>,
}

/// Runs the failing `input` through `programs` again in the debug
/// interpreter, traced as `how` says, and finds what explains it.
pub fn retrace(
    source: &str,
    programs: Programs,
    input: &str,
    how: Option<&Result<Explaining, String>>,
    explain: bool,
    stack_limit: Option<u32>,
) -> Retraced {
    let config = |assignments: bool| TraceConfig {
        source: source.to_string(),
        assignments,
        record_frames: true,
        ..TraceConfig::default()
    };
    let observe = |program, input: &[u8], config| {
        observe(program, input, stack_limit, config)
            .recording
            .unwrap_or_default()
    };
    let alone = |fast: Recording, divergence| Retraced {
        fast,
        slow: None,
        stages: Vec::new(),
        divergence,
    };
    match (how, programs.interactor, programs.brute) {
        (Some(Ok(Explaining::Answer)), Some(interactor), _) => {
            let sent = crate::duplex::talk(
                programs.fast,
                interactor,
                input.as_bytes(),
                TRACE_LIMIT,
                stack_limit,
            )
            .sent;
            let fast = observe(programs.fast, &sent, config(explain));
            let divergence = explain
                .then(|| answered_divergence(source, &fast))
                .flatten();
            alone(fast, Ok(divergence))
        }
        (Some(Ok(Explaining::Stuck)), ..) => {
            let stuck = TraceConfig {
                max_steps: Some(STUCK_STEPS),
                stop_when_full: true,
                ..config(false)
            };
            let fast = observe(programs.fast, input.as_bytes(), stuck);
            let divergence = stuck_divergence(source, &fast).map(Some).ok_or(format!(
                "the solution never finishes, and none of its first {STUCK_STEPS} steps repeats an earlier one"
            ));
            alone(fast, divergence)
        }
        (Some(Ok(Explaining::Versus)), _, Some(brute)) => {
            let fast = observe(programs.fast, input.as_bytes(), config(explain));
            let slow = observe(brute, input.as_bytes(), config(explain));
            let stages = stages(&fast, &slow)
                .into_iter()
                .map(|s| Stage {
                    name: s.name,
                    line: s.line,
                    fast: s.fast,
                    brute: s.brute,
                })
                .collect();
            let divergence =
                crate::calls::first_call_divergence(source, &fast, &slow).or_else(|| {
                    explain
                        .then(|| explain_divergence(source, &fast, &slow))
                        .flatten()
                });
            Retraced {
                fast,
                slow: Some(slow),
                stages,
                divergence: Ok(divergence),
            }
        }
        _ => alone(
            observe(programs.fast, input.as_bytes(), config(explain)),
            Ok(None),
        ),
    }
}

fn observe(
    program: &cpl_ir::Program,
    input: &[u8],
    stack_limit: Option<u32>,
    config: TraceConfig,
) -> cpl_interp::Observed {
    let mut sink = Vec::new();
    let io = cpl_interp::Io::fixed(input).until(cpl_interp::Stop::after(TRACE_LIMIT));
    match stack_limit {
        Some(bytes) => cpl_interp::observe_on_this_thread(
            program,
            io,
            &mut sink,
            cpl_interp::Limits {
                stack_bytes: bytes as usize,
            },
            Some(config),
        ),
        None => cpl_interp::run_observed(program, io, &mut sink, Some(config)),
    }
}
