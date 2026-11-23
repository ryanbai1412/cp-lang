# Debugger

The debugger is everything a contestant uses between "this is wrong" and
"now I know why": finding an input that breaks the solution, seeing what the
program did on it, and checking the fix. Each surface below answers one
question, and they hand a case from one to the next, so the whole loop is
one workflow, not five tools. The loop itself is
[wrong-answer.md](../workflows/wrong-answer.md).

**Stress** (`cpl stress`) answers "is there an input where I am wrong?". It
generates inputs from the program's own `read()` types, runs the solution
against a brute force, a checker or an interactor, shrinks the first
failure to the smallest input that still fails, and saves it as a test. With
no brute yet it still finds crashes and time-outs. Doc:
[stress.md](../cli/stress.md), use cases:
[stress-usecases.md](../cli/stress-usecases.md).

**Stress `--explain`** answers "which line is wrong?". On the
shrunk case it runs the solution and the brute side by side and names the
first value they disagree on, then the lines that made it. A contestant
should read the planted line off the report without opening a trace.
Today it names the planted line for every seeded bug in `bench/stress/`.

**Traces** (`cpl run --trace`, the HTML viewer) answer "what did it do?".
Every change to a variable, every print and every call on one input,
shown in the program's own names and values, filtered to the functions,
tests and variables that matter. A trace of the case stress saved last is
one command.

**Stepping** (`cpl dap`, the editor's debugger) answers "what is true
here?". Breakpoints, stepping in, over and out, and the locals of every
frame, inside the editor the contestant already uses.

**Replay** answers "how did it get here?". Stepping backwards as well as
forwards, to the step that last changed a value, over a recording of the
whole run, at most 20% slower than an untraced run. It is milestone 1.5.

Owner: debugger. How the debugger is built and checked is in
[handbook/domains/debugger.md](../../domains/debugger.md) and
[handbook/design/debugger.md](../../design/debugger.md).
