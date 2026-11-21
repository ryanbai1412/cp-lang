# agc079c

Score: 3/5
Dream: pending cold rescore (rewritten: proves unique parity relation + minimal reversals in header, adopts `std.dsu.ParityDsu` instead of hand `comp` walk, extracts `pop_top_front(x)` to remove duplicated `runs.last!` arms)

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

## Cold review (HEAD)

Dream 3/5 (new); sent back.
- Pass 1 (lines 36-69) is the C++ run-stack machine: `runs.last!` five times, `pop_front` then re-read `runs.last!.vals.first`, and two nearly identical `rel[i] = Rel(runs.last!.vals.first, false)` arms (lines 48, 67).
- The header asserts a "unique parity relation" per component and that reversing only when forced is optimal (lines 3-9), but derives neither.
- `comp` (lines 71-77) is a hand-rolled parity-DSU path walk; `PotentialDsu` is additive, so it is a real gap, but it isn't in the friction report.
- `LinkedList` (line 18) isn't in the stdlib; the friction report asks for it.

Port 3/5: the dream's structure with `head`/`tail` runs in place of `LinkedList` (reported gap).
