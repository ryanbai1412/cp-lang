# agc079c

Score: 4/5
Dream: 4/5

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

## Cold review (HEAD)

Dream 3/5, rescored after 7042fdf4; sent back. `ParityDsu` replaces the hand component walk and `pop_top_front` removes the duplicated `runs.last!` arms.
- `LinkedList` (lines 24, 55) is still not in the stdlib (E0201, E0204); the friction report covers it.
- Pass 1 (lines 52-78) is still the C++ run-stack machine. The header (lines 6-10) restates its branches but never says what a `Run` is or what `opp_below` means, so the `return none` on line 62 is not derived.
- Owed: define a run and `opp_below` as an invariant in the header, then show why each branch of lines 57-78 keeps it.

The port score (3/5) holds; the port was not touched.

## Cold review (HEAD)

Dream 3/5 → 4/5, rescored cold on current main. The header now states the `Run(vals, opp_below)` invariant and shows that every branch of `pop_value`, and each forced `return none`, preserves it. The run stack is the algorithm, not a translation.
- The only error is `LinkedList` (lines 47, 104), which is reported.
- Minor: the minimality argument for Pass 2 (lines 34-37) is asserted. The two `match active` blocks (lines 124-132) choose an end by hand; a side-taking `push`/`pop` on `Deque` would say it once.

## Cold review (HEAD)

Port 3/5 → 4/5, rescored cold on current main; dream 4/5 holds. The port checks with zero errors. It adopts the dream's `Run` invariant header (lines 5-31), `ParityDsu<Val>` (line 58), `pop_top_front`/`pop_value` and §4.9 `and` guards (lines 81, 104, 128). The hand `pred`/`succ` chain (lines 55-64, 72, 86) is the reported `LinkedList` gap.
- Minor: as in the dream, the Pass 2 minimality argument (lines 33-36) is asserted, and the two `match active` blocks (lines 125-132) pick a deque end by hand.
