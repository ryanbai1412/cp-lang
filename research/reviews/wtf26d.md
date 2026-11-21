# wtf26d

Score: 2/5
Dream: 2/5

Rescored cold on main 1a5cc751 (old 2/5, new 2/5) for accepted syntax (decisions.md §53, §58, §63, §70) and stdlib reuse. Scored on its first landing (ddf13f69); no dream commit since. Syntax: nothing outside decisions.md (`.inverse()` is a method, not a getter). The nested `if let ... else` optional clamps (lines 52-53) are hand-folding a `min` over optionals. Sent back: under 4, the lane owes a rewrite before the port is judged.

Re-reviewed after ddf13f69. `Pos`/`Val` domains and a named `CondStack` replace the raw ints and the triple pop loop, but control flow is still the C++: counters, `while true`/`break`, a predicate that mutates globals and index shifts. The dream is under 4, so this score is provisional. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Header (lines 1-5) restates the mechanics, not why the greedy is optimal.
- Hand inverse (lines 13-14): `p.inverse()` exists (the dream uses it).
- Hand search with a counter (lines 16-21): `st` is `Pos.find(i => p[i] != q[i])`.
- Hidden side effects (lines 47-71): `try_place` returns `Bool` but writes `used`, `conds` and `seen_inversion`; it is called in a retry loop (lines 77-85).
- Flag loop (lines 80-86): `while true` / `break` around a hand descending search for the next unused value (lines 81-83).
- Index shifts (lines 74-76, 101-102, 113-114, 123): `Pos(idx + 1)`, `Pos(idx + 2)` and `.ord` round-trips.
- Raw `Int` block bookkeeping (lines 88-124): `block_starts`, `p_to_block: [Pos]?Int`, `last_thresh = n`.

## Owed rewrites

- `p.inverse()`; `Pos.find` for `st`; split `try_place` into a pure `cap_for(i)` and an `apply`; `Val.find_last(v => v < q[i] and not used[v])`; give blocks a domain.

## Language gaps

No friction report exists. None needed.

## Dream notes

Scored cold (new dream in ddf13f69) against `design/EXEMPLARS-PORTS.md` and `design/sketches/ioi2025/`.

- Hidden side effects (dream lines 43-66): `try_place` mutates `used`, `conds` and `seen_inversion` behind a `Bool`, called in a retry loop (lines 72-77).
- Flag loops (dream lines 41, 64, 75-79): `seen_inversion`, `while true` / `break`, `break` out of the outer loop.
- Accidental complexity (dream line 22): `pref_min` rescans `Val.first..=v` per `v`, O(n²) at n = 2e5; it is a prefix min.
- Index shifts and raw ints (dream lines 69-73, 82-110): `filled = i.ord + 1`, `Pos(pos + 1)`, `block_starts: List<Int>`, `last_thresh = n`.
- `?? n` sentinel on a `Pos` search (dream line 18).

Sent back: the dream goes to its port's lane (the polish lane if that lane has moved on) to rewrite before the port is judged.

## Rewrite note (pending rescore)

Rewritten dream and port: O(n) `pref_min` scan (fixed O(n²) dream bug), split side-effecting `try_place` into pure `forms_inversion`/`required_cap`/`can_place` + mutating `commit_place`, removed `while true` retry loop and `?? n` sentinel on `Pos`, and iterated `Pos` directly in the dream's block-draining loop. Emitted C++ grew `20,980 -> 22,240 B` (+1,260 B, 34% of 64 KB) from the four small local helper functions replacing `try_place` (codegen: inline small single-call helpers).
