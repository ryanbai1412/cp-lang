# wtf26d

Score: 2/5
Dream: 3/5

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

## Cold review (HEAD)

Dream 2/5 (was 2), sent back. The first phase improved: `forms_inversion`/`required_cap`/`can_place` are pure, and `commit_place` is the one mutator. The block packing (lines 92-120) is still hand-rolled bookkeeping:
- `block_starts: List<Int>`, `p_to_block: [Pos]?Int`, the `en` counter and the `?? (block_starts.len - 1)` default (line 118) hand-roll "sort the remaining values by (block, descending value) and write them from `filled`". Compute each value's block key, then `sorted(by:)` and one zip into `q[filled..]`.
- `pref_min` (lines 42-46) is a prefix min over optionals written as a loop; `scan` with `min` over present values says it.
- `seen_inversion` (line 49) and `filled` (line 80) are flags carried across helpers; `filled = i.ord + 1` is written twice (lines 85, 89).
- `(print(q); return)` (line 37) doesn't parse (E0103), and `Val.last(pred)` (line 87) is a value, not a function: write `Val.rev.first(pred)`.
- The header doesn't derive why the stack thresholds order the suffix blocks (lines 6-8).

## Cold review (HEAD)

Dream 2 → 3/5, rescored after 9aa24cb4; sent back. The header states the greedy and the monotone-stack invariant, and `CondStack` with `.somes.min()` replaces the nested optional clamps.
- `?? (print(q); return)` (line 35) isn't CP Lang (E0103/E0102).
- The block partition (lines 77-105) is hand bookkeeping: a nested drain that mutates `cond.max_p`, a manual `min_p` scan with `break`, then `p_to_block` and `block_cursor` arrays. Placing values into blocks in decreasing order is `group(by:)` over the block id followed by one sorted fill.
- `seen_inversion` is mutable global state read inside `forms_inversion` and written in `commit_place` (lines 41-62), so `can_place` isn't pure.
- The header claims, without proof, that the blocks are disjoint and that filling them in decreasing order is optimal (lines 11-14).

The port score (2/5) holds.

## Cold review (HEAD)

Dream 3/5, rescored after f69a1ce4; sent back. `can_place` is now pure, the invalid `?? (print; return)` is gone, and the suffix fill is a `sorted` plus `zip`.
- It fails to check. `pop_while` (line 28) is a reported gap, but `.somes` (line 66) and `take_while` (line 112) are not in the stdlib and have no friction item. Lines 112 and 121 index `[Pos]` arrays with `Int` (E0301).
- The block partition (lines 100-117) is still hand bookkeeping: `block_of` with an `n` sentinel (line 102), a `block_id` counter and `last_thresh`. Line 112 is a 150-character chain. The header (lines 13-17) doesn't say why `min_p` is the block's left end.

The port score (2/5) holds; the port was not touched.
