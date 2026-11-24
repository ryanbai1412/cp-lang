# abc481g friction

## Redundant min/max on an ordered pair
- **Where:** `port.cp` built events from `let (a, b) = cakes[i]` with `min(a, b)` and
  `max(a, b)`. `dream.cp` uses `cakes[i].a` and `cakes[i].b`.
- **What felt wrong:** the statement guarantees a_i < b_i, so `min`/`max` always returned `a`
  and `b`: a check that always passes (idiomatic-code/R13).
- **Goal it hurts:** express.
- **Ideal code:** the dream's `events.push(Event(cakes[i].a, End.Low))`.
- **General fix:** at the source: the port takes the dream's lines. With that edit, `cpl test`
  passes 8/8 and `corpus.py verify abc481g` has 0 findings. The dream and the current port
  print the same output at `--max` (0.176 s each).
- **Severity:** low.
- **Owner:** idioms (this port); the edit lands in its own `port(abc481g)` commit.

## The order a_i < b_i is not a type
- **Where:** `let cakes: [I](a: 1..=998_244_352, b: 1..=998_244_352) = read()`.
- **What felt wrong:** the bound `a < b` lives only in the `Problem:` header.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `let cakes: [I](a: 1..=998_244_352, b: a + 1..=998_244_352) = read()`, a
  field bound that depends on an earlier field, as `let r: l..=1e18` does for two reads.
- **Run:** `let c: [2](a: 1..=9, b: a + 1..=9) = read()` under `cpl run` (09954f019):
  `error[E0201]: no 'a' in scope`.
- **General fix:** dependent ranges between fields of one record read (core types).
- **Severity:** low.
- **Owner:** Core (types).

## Measurements

- Judge limits: 4000 ms, 1024 MB, as the port's `limits:` line says; checked against the AtCoder task page, https://atcoder.jp/contests/abc481/tasks/abc481_g.
- Tokens: port 761, original 1,402.
- `--max` (best of 10): port 0.199 s / 20 MB, accepted C++ 0.266 s / 17 MB. Same output.
- Emitted C++: 26,576 bytes.
