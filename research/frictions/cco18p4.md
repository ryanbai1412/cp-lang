# cco18p4 friction

## The corpus suite cannot run an interactor

- **Where:** `cpl-conform corpus cco18p4`. `crates/cpl-conform/src/corpus.rs` feeds
  `tests/NN.in` as stdin and compares stdout with `tests/NN.ans`; it never runs the problem's
  `interactor.py`. So the tests are transcripts: `NN.in` is what `interactor.py` sent to
  `spec.cp` on the hidden test `tests/NN.case`, `NN.ans` what `spec.cp` printed. They pin
  `spec.cp`'s probe order: a correct rewrite that asks other cells reads `wrong` until the
  transcripts are regenerated.
- **Repro:** `interactor.py tests/01.case -- <AC>` passes (the AC asks `? 1 1 / ? 1 4 / ? 1 6
  / ...` and prints `! 8`); the AC run on `tests/01.in` gets the replies to `spec.cp`'s
  `? 1 1 / ? 1 5 / ? 1 8 / ...` and prints `! 9`.
- **Goal it hurts:** dependable evidence (an interactive port is gated on its dialogue, not
  its verdict).
- **General fix:** when `corpus/problems/<id>/interactor.py` exists, the corpus runner runs
  `interactor.py tests/NN.case -- <interpreter>` for each `NN.case` and passes on exit 0.
  Routed to spec (owner of `crates/cpl-conform`).
- **Status:** open.

## `@memo` on a function that does I/O checks clean

- **Where:** the old `spec.cp` memoized `fn ask(x, y) = wire.ask(...)` with `@memo(map)`.
  R06.4.1 says a memo function may not do I/O, but `cpl check` reports no errors.
  `spec.cp` now caches scores in a `HashMap` by hand.
- **Repro:** `@memo(map)` / `fn f(x: Int) -> Int = read<Int>() + x` / `print(f(1))` is
  "no errors" (same with `print(x)` in the body).
- **General fix:** the checker's memo purity check treats `read`/`print`/`Wire` calls (and
  calls that reach them) as I/O and rejects them with the rule. Routed to compiler.
- **Status:** open.

## `cpl run --release` cannot talk to an interactor

- **Where:** `interactor.py tests/08.case -- cpl run --release spec.cp` hangs; the interpreter
  and the `cpl build` binary (g++ -O2) pass all 13 cases.
- **Repro:** `let wire = Wire()` / `let x: Int = wire.ask("?")` / `print(x + 1)` under
  `(sleep 3; echo 5) | cpl run --release w.cp` prints `?` only once stdin ends.
- **General fix:** `cpl run --release` streams stdin and stdout to the compiled binary.
  Routed to tools.
- **Status:** open.
