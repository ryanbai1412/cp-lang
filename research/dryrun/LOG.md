# Contest dry run log

Default `cpl` (no `CPL_SPEC`), solutions written from the user docs only.
Each problem has `<p>.cp`, `<p>.tests/`, a C++ reference `<p>_ref.cpp` and a
generator `<p>_gen.py`. `verify.sh <p>` builds for `--judge codeforces`,
compiles the emitted C++ with `g++ -O2 -std=c++17` and compares it with the
reference on random inputs.

## Problems

- p01_ordinary (CF 1520B, multitest math): about 1 min, solved. check, run,
  test, stress (200) and verify (200) all pass. No friction.
- p02_movies (CSES 1629, greedy + sort): about 2 min, solved. All checks
  pass. No friction.
- p03_forest (CSES 1652, 2-D prefix sums via `prefix([Row, Col] ...)` and
  `.sum(rows, cols)`): about 2 min, solved. All checks pass. No friction.

## Frictions

- P2, tooling: `cpl judges --help` treats `--help` as a file name:
  "cpl: can't read `--help`: No such file or directory (os error 2)".
  Expected: usage text, like every other subcommand.
- P2, docs: the brief points to `plan/1x-*.md`, but no such files exist;
  the language-as-implemented pages are `plan/10-*.md` to `plan/19-*.md`.
