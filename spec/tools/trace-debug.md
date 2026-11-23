# Traces, the debugger and runtime errors

Status: draft for 0.3.0. Sources: `plan/50-debugging.md`, decisions 41,
43.

- **Traps** (checked builds) print the rule, the place and local values
  in source terms (positions in the file's base, ranges by name), plus
  the call trace. Output written so far is flushed first (Rtrace.1).
- **`cpl run d input --trace`** writes one HTML file: every top-level
  statement and part call with its inputs and result, each read token
  with the value it became and the line that consumed it, and
  collapsible loops. Quiet by default: no per-iteration rows unless a
  filter asks (`--at line`, `--var name`, `--when expr`) (Rtrace.2).
- **Stage snapshots**: `cpl snap` saves a part's inputs; `cpl run d
  --stage snap` reruns only that part while iterating (Rtrace.3).
- **`debug(x)`** prints with a source location in checked builds and is
  removed from submissions (Rtrace.4).
- **The debugger** (the editors' debugger over `cpl dap`) runs on the
  interpreter: breakpoints, conditional breakpoints, step into
  generators and closures, watch expressions in the language, and
  reverse step over the recorded run (Rtrace.5).
- Interpreter and C++ agree on every program, including on input they
  reject; the differential tests `scripts/check.sh` runs check it (Rtrace.6).
