# Friction Report: cco20p5

## 1. Offline segment tree over time (`TimeSegTree<Q, T>`)

- **Area:** stdlib
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_36/dream/cco20p5.cp:42,51,55,80-103`
- **Port:** `proto/ports/scout_batch_36/cco20p5.cp:42-50,101-131`

Dynamic-connectivity / add-remove-over-time problems map each element's `[start, end]` lifetime onto `O(log q)` nodes of a segment tree over time and DFS with rollback. Because `stdlib/` has no `TimeSegTree`, the port hand-rolls `seg[o * 2]` and `dfs`.

## 2. Rollback-capable prefix-max `Fenwick` (`RollbackFenwick<Coord, Max>`)

- **Area:** stdlib
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_36/dream/cco20p5.cp:57-64,101-102`
- **Port:** `proto/ports/scout_batch_36/cco20p5.cp:68-85,127-129`

`std.fenwick.Fenwick` supports additive groups (`+`/`-`) without rollback; prefix-max with checkpoint/rollback forces a hand-written `i -= i & -i` / `j += j & -j` tree and `log: List<Undo>`.

## 3. No `HashMap.entry` (default-insert) in `cpl run`

- **Area:** stdlib
- **Goal:** express
- **Severity:** low
- **Dream:** `proto/ports/scout_batch_36/dream/cco20p5.cp:48` (`active.entry(key).push(i)`).
- **Actual:** `proto/ports/scout_batch_36/cco20p5.cp:58-59` writes `if not active.contains(key): active[key] = []` before each push.
- **Error:** `E0203: a HashMap<(Int, Int), List<Q>> has no method entry that cpl run knows`.
- **Ask:** one default-insert spelling on `HashMap`/`Map` (`entry(k)` or a `default:` constructor option), so accumulating into a keyed list is one line.

## 4. An enum with domain-element payload fields doesn't read by tag

- **Where:** `enum Op = @tag("A") Add(l: X, r: X) | @tag("R") Remove(l: X, r: X)` over
  `domain X = 1..=1e6`; `read<[q]Op>()` on `A 1 5` is R0501 "expected a number, found `A`"
  in `cpl run`, and the C++ build stops with "something else where a number was expected".
  With `Int` or `1..=1e6` fields both engines read it (R10.2.3).
- **Repro:** `domain X = 1..=10` / `enum Op = @tag("A") Add(l: X, r: X) | @tag("R") Remove(l: X, r: X)`
  / `for op in read<[1]Op>(): match op: ...` on input `A 1 5`.
- **Goal it hurts:** express (the statement's bound is written twice: once as the domain the
  segment tree is over, once as the payload range).
- **General fix:** the tagged-enum reader reads a domain-typed payload field like `read<X>()`.
  Already routed to compiler (`plan/domains/compiler/worklog.md`, from noi02p1).
- **Workaround:** payload fields are `1..=1e6` and the segment tree is indexed with `X(p)`.
- **Status:** open (compiler).
