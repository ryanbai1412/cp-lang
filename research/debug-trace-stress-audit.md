# Debugging, tracing and stress testing: end-to-end audit (0.2.2)

Run on `main` at `aaaa4fa` with the debug build of `cpl`, on copies of the
repository's own examples in a scratch directory. The examples are
`crates/cpl-cli/tests/debugger/*.cp` (planted bugs, a `@checker` and an
`@interactor`), `proto/tests/cheatsheet/s8_*.cp` and
`proto/tests/fn/range_sum*.cp`. `cpl bench` was also run on 40 ports from
`proto/ports/`.

## What works

`cpl stress` finds and shrinks planted bugs fast. `cf2284a2_planted.cp`
(`@brute(main)`) fails on the first case in 0.12 s, shrinks from 24 numbers
to 4, and shows exactly where the solution and the brute diverge:

```
  FAIL on case 1 (seed 1): WA
  shrunk from 24 numbers to 4:
  that is: n = 3  a = [0, 0, 1]
  first difference at line 1, token 1: expected `4`, got `3`
  debug(...) both ran, solution | brute:
  ! line 17: case 1 · debug piles: 3, 1 | 4, 4
    ^ the first one that differs
  trace cf2284a2_planted.tests/stress-1.trace.html
  saved cf2284a2_planted.tests/stress-1.in
```

That is well inside the P0 target of under 1 s to the first case. The
`debug(...)` diff between the fast and slow runs is the strongest feature of
the toolchain.

Other passes:

- **Clean files:** `cf2284b.cp` (`@checker`, 1.24 s), `guess_multitest.cp`
  (`@interactor`, 1.11 s), `s8_brute_main.cp` (0.92 s) and `s8_checker.cp`
  (1.17 s) each pass 1000 cases. 10000 cases of `cf2284b.cp` take 12.5 s.
  With `--jobs 1`, 1000 cases take 5.5 s, so the default parallelism gives
  4.4x.
- **Regression cases:** the saved case is picked up by `cpl test`. On
  `cf2284a2_planted`, `stress-1 WA` fails in 1 ms with the same diff.
- **Traces:** `--trace` writes a self-contained trace page of 93–94 KB, with
  the solution and the brute side by side. `cpl run --trace` writes the same
  viewer in 16 ms, and `--format json` gives the schema-versioned trace.
- **Editor output:** `cpl stress --format json` returns the counterexample,
  the raw input, the verdict and the `debug` stage that first differs.
- **`cpl run --release` and `--brute`:** both work, each in 15–17 ms.
- **Bench passes:** 23 of 40 ports pass, and the output gives per-shape
  time and memory. For example, `ccc20s2` takes 77–96 ms and 63 MB at
  1,000,002 numbers. The slowest pass, `cco24p1`, took 19.3 s of wall time.

## Bugs and UX gaps, most important first

1. **A runtime error under `stress` or `test` is just "crashed".**
   `cpl stress s8_interactor.cp --gen ./gen3.sh` reports `RE … the solution
   failed: crashed`, and `cpl test` says `RE 26 ms crashed`. On the same
   input, `cpl run` gives the real cause with a source span: `error[R0001]:
   the input ended where a number was expected` at `ask("?", n, limit: 2)`.
   Both should print what `cpl run` prints.
2. **`cpl bench` reports a runtime error without a reason, and says it is
   "over the limits".**
   - `noip14p2` gets `RE` on all 6 shapes at 200,001 numbers, after 80–99 ms
     and 13–15 MB. Nothing says whether it was stack overflow on the deep
     generated tree, a failed assertion or a bad input.
   - The summary line "6 shapes are over the limits" is wrong for a crash.
   - The JSON `runs` entries have no `error` field.
3. **`cpl bench` quietly generates half an input.** `dpg` reads its edges
   with an untyped `Graph<V, E>.directed(read())`. Bench writes only `n m`
   ("2 numbers") and reports `RE` on every shape. It should refuse, the way
   it does for `Pt` and `Road` ("can't generate a `Pt` yet"), and name the
   `read()`.
4. **"no typed `read()` to make inputs from" when there are typed reads.**
   Seen on:
   - `cpl bench` for `cf2284b.cp` and `cf2284a2_planted.cp` (`@!multitest`
     files with `let n: 1..=6 = read()`);
   - `noip18p2` (its reads are inside a block);
   - `noip09p2` (a tuple read, `let (a0, a1, b0, b1): (1..=2e9, ..) = read()`);
   - `cpl stress` for `s8_interactor.cp`, which has `let n: 1..=5 = read()`
     but reads it from the interactor.

   The message should say which `read()` it could not use and why (a
   multitest header, a nested scope, a tuple, an interactor-fed read, or
   `Str` with no length bound as in `noi14p4`).
5. **A `@brute` on a fn other than `main` is ignored by `cpl stress`.**
   `range_sum_bug.cp` has `@brute(range_sum)` on `range_sum_slow`, and stress
   answers "there is nothing to compare range_sum_bug.cp with". Stress should
   compare the two functions on random arguments, or the error should say that
   only `@brute(main)` is used. The cheat sheet documents only
   `@brute(main)`, so the file may simply be stale.
6. **`--gen` only takes a file.** `--gen "python3 -c 'print(3)'"` fails with
   "there is no file python3 -c 'print(3)'"; a one-line generator needs a
   script file. Accepting a command line (split like a shell) would match how
   `--brute` and `--checker` are usually passed.
7. **The checker error does not say which file ended.** For
   `cf2284b_planted.cp` the `@checker` reports "the input ended where a
   number was expected (input line 3, column 1)". The checker was reading the
   solution's output, which had too few numbers. It should say "the output
   ended", with the line in the output.
8. **`cpl run --trace slow` on a run where `slow` never executes** writes a
   1-step trace without a warning. It should say that `slow` was never called.
9. **7 of the 40 ports `cpl bench` was run on no longer build on `main`:**
   - `nccc10s2` uses `Perm`;
   - `ioi05p6` uses `WithMin`;
   - `noi01p5` uses `product`;
   - `ccc03s4` uses `StrHash`;
   - `ccc21s4` still has a `where` guard;
   - `ceoi09p3` hits `Compressed`;
   - `noi02p1` has a `read()` with no input format.

   These are stale ports for the friction-lint lanes, not bench bugs.
10. **Bench's generator cannot make records yet.** `Pt<F64>`, `Pt`, `Road`
   and `Stroke` stop 4 of the 40 ports with "can't generate a … yet". Records
   of typed fields are the next generator gap.

## Not covered

- The VS Code / LSP side of live counterexamples.
- The trace viewer opened in a real browser: the pages were checked for size
  and content only.
- `cpl bench` on interactive problems.
