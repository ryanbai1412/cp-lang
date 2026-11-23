# I have WA and want to know why

> 70 minutes into a Div. 2 round. C passes the samples and gets WA on
> test 9. You don't know whether the idea is wrong or one line is.

This is the debugging loop of journey J7: run, stress, shrink, trace,
edit, rerun. Each step hands its case to the next, so the contestant never
copies an input or retypes a command line. Owner: debugger.

## The walkthrough

1. **Run.** `cpl test c` runs the samples: all pass. Nothing to fix yet.
2. **Stress with what you have.** `cpl stress c.cp` with no brute: a case
   fails only if it crashes or runs too long. One does,
   `n = 1  a = [0]`, with `index 1 is outside an array dimension of length 1` at
   line 5 and the locals. If nothing fails, go on.
3. **Write the brute.** A few lines that try every option, in the same
   file as `@alt(program) fn brute()`. `cpl stress c.cp` again.
4. **Read the counterexample.** The failure is shrunk to the smallest case
   that still fails, shown raw and by name, with both answers, the first
   wrong token and the line that made the first wrong value. It is saved
   as `c.tests/stress-4.in` with the brute's answer beside it.
5. **Trace it if the line isn't enough.** The report's next step,
   `cpl run c.cp --trace`, records that saved case and opens it in the
   viewer, in the program's own names.
6. **Edit and rerun.** Fix the line. `cpl test c` reruns the samples and
   the saved case: green. `cpl stress c.cp` again: `1000 cases agree`.
7. **Submit.** `cpl submit c.cp` builds the C++ and runs every saved case
   on it.

**What it costs today.** Steps 1 to 7 work, and each report names the next
command. Three steps cost more than the ideal: step 4 names the line only
with `--explain`; step 6 is two commands where one should do; and when the
input is a shape stress can't make, step 2 needs a `@gen fn` first. Each is
a use case in [stress-usecases.md](../cli/stress-usecases.md).

**Off the happy path.** Many answers are right: a `@check fn` replaces the
brute at step 3. Interactive: the file's `@interactor` judges each case. The
solution hangs: the case fails as a time-out and the explanation names the
loop that stopped changing. The brute is wrong: stress blames the brute.
Stress finds nothing but the judge still says WA: the bug lives at sizes
the default `--size 8` never reaches, such as an overflow at the bounds.
`cpl lint` names a witness input for each bound it can't prove, and
`cpl bench` runs the maximum size (use case `stress/U28`). A helper fails
(the checker, interactor or `@gen fn`): stress should blame it by name, as
it blames a brute (`stress/U30`).

## Acceptance

`crates/cpl-cli/tests/wrong_answer.rs` runs the walkthrough on one program
with a planted bug: stress with no brute finds its crash, stress with a
brute finds its wrong answer, the report names the next commands, the trace
records the saved case, and after each fix `cpl test` and `cpl stress` are
green. `crates/cpl-cli/tests/debugger.rs` runs the same loop through a
checker, an interactor and a machine with no C++ compiler.
