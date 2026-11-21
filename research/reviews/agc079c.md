# agc079c

Score: pending (was 2/5: added dream, stated time-reversed parity-component invariant in header, domain Val/Pos, enum Side, Rel/Run structs, solve() -> ?Int, single face() helper; emitted C++ grew 18,275 -> 22,223 B (<48 KB) from Side enum, Rel/Run structs, and Val/Pos domains)
Dream: pending (added dream using LinkedList<Val> with O(1) splice_back, Side enum, Val/Pos domains, and face() helper)

The header states the two passes, but pass 1 is a hand linked-run stack with `pred`/`succ` arrays and pass 2 a flag-driven deque replay. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Header (lines 1-5) describes the mechanics, not the invariant: why a run must enter on a given side and why the reversal count is minimal.
- Raw index shift (lines 10-11): `perm[i] - 1` into `[0..n]`; a `domain Val` with `read(base: 1)` says it.
- Hand linked list (lines 13-14, 19-23, 36, 50, 62): `pred`/`succ` arrays maintained beside the `runs` stack.
- Flag state (lines 6, 55-60, 76-90): `neq`, `at_front` and nested `!=` on Booleans encode side relations.
- Duplicated flip-and-count (lines 82-84 and 87-89).
- Repeated early exits (lines 42-44, 47-49) and `continue` control flow (lines 39, 53).

## Owed rewrites

- State the invariant; `domain Val`; one `turn()` helper; name the side relation as a parity DSU or a typed `Side` relation.

## Language gaps

No friction report exists. None needed.
