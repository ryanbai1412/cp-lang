# Stress, shrinking and laws

Status: draft for 0.3.0. The library side is `lib/random-stress.md`.

`cpl stress d` builds the checked program, finds every `@alt`, `@gen`,
`@check` and `@interactor`, and runs rounds:

1. Generate an input from `@gen` or from the program's reads.
2. Run the program and each applicable alt (whole-program alts compare
   output; inner alts are substituted at call sites and compared there).
3. Apply the relation and unary checkers; sample laws of operation
   values the run built; for `@cache` fields, rerun with caches cold and
   compare.
4. On failure, shrink the recorded `gen` choices to a minimal input that
   still fails, then report it, each implementation's output, the first
   differing line, and the first part whose alt disagrees, with values in
   the file's base. Save it to `d.tests/stress-<seed>.in`.

Options: `--time`, `--threads`, `--seed`, `--max-size`, `--laws=off`.
Runs are deterministic per seed and parallel across threads. `cpl
locate` is stress plus part-level bisection on a given failing input.
