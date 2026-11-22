# Friction Report: ioi13p3

## 1. Stateful `@grader` object (IOI interactive/grader API)

- **Area:** core
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_52/dream/ioi13p3.cp:46-79`
- **Port:** `proto/ports/scout_batch_52/ioi13p3.cp:48-84`

IOI grader problems expose multiple entrypoints (`init`, `changeH`, `changeV`, `escape`) sharing mutable state whose domains (`Block = 0..ceil_div(r, B)`) depend on arguments passed to `init`. Today CP Lang has no `@grader struct` state object, so runnable ports use a stdin command loop instead of `@grader` methods.
