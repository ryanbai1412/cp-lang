# Friction report: ioi15p4

## Dream

The port as written, with `@grader` on the three judge functions `init`, `updateX` and
`updateY` instead of the local driver that reads the sample grader's input.

## The grader task has no local judge for its functions

- **Where:** the stdin driver at the end of `port.cp`.
- **What happened:** see `research/frictions/ioi18p1.md` and `research/frictions/ioi22p5.md`: a
  `@grader fn` with a local driver builds without `main`, and an `extern fn` has no local stub.
- **General fix:** keep the top level as the local driver of a `@grader` file.
- **Severity:** low.

## Measurements

Contestant-written tokens: port 343, original 824 (`corpus/audit.py` count). Emitted C++: 32553 bytes (`cpl build`, judge `generic`).
The accepted C++ is a grader function with no main, so the max input is timed only by `corpus/generators.py`.
