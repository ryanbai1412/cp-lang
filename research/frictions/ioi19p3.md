# ioi19p3 friction

## A row or column view of a grid loses its key domain

- **Where:** `h[r].valleys()` and `h[.., c].valleys()` on `h: [R, C]Int` yield `Range<Int>`, so a
  `Map<Range<C>, Int>` keyed by them is E0302. R04b.1.12 says a row keeps the column domain.
  The spec copies each line out (`[C] c => h[r, c]`), O(m) per row.
- **Repro:** `domain R = 0..2; domain C = 0..4; let h: [R, C]Int = [R, C] (r, c) => c.ord; let p: ?C = h[R(0)].argmin()`
  is E0302 "expected `?C`, found `?Int`".
- **Goal it hurt:** express (a grid's rows are the natural sequences here).
- **Routed:** compiler worklog.

## `cpl check` never checks a fn nothing calls

- **Where:** the old spec was one `@grader fn` with stale calls (`m.get_or_insert(k)` without
  `v`, `s.l.r!` on an `Int`); `cpl check` said "no errors" because it skips the body of any fn no
  code calls, and a `@grader fn` is called only by the judge.
- **Repro:** `fn f() -> Int:` / `    var m = Map<Int, List<Int>>()` / `    m.get_or_insert(3).push(1)` /
  `    m.len` checks clean; add `print(f())` and it is E0303 "`get_or_insert` needs the argument `v`".
- **Goal it hurt:** hard to get wrong (grader entry points go unchecked until the judge).
- **Routed:** compiler worklog.

## A `Range<D>` is not a hashable `Map` key in C++

- **Where:** valleys keyed by their range (`Map<Range<C>, Int>`) run in `cpl run` but the C++
  fails g++; the spec keys by `(s.first!, s.last!)` instead.
- **Repro:** `domain C = 0..3; var m = Map<Range<C>, Int>(); m[C(0)..C(2)] = 1; print(m.len)`
  prints `1` in `cpl run`; `cpl build` + g++ is "no match for call to `(cpl::CpHash) (const cpl::Range<int>&)`".
- **Goal it hurt:** dependable submission (interpreter and C++ disagree on a checked program).
- **Routed:** compiler worklog (beside the `Bitset<D>` key report).

## No local stub for `@grader fn`

- Already routed (tools worklog, apio23p1). The spec reads the sample grader's stdin format.
