# cco24p2

Score: 3/5
Dream: 3/5

Cold review against the friction's `## Dream`; 3 for the hand-undone search (reported) and the raw-`Int` recursion (not). The count check (lines 29-34), the two-flavour case (lines 35-37) and the distinct case's patience piles (lines 38-48) follow the header. The N <= 10 search (lines 49-83) keeps top-level state and undoes each step by hand, which the friction reports with a `cpl run` of the dream line (E0201, no `matchings`).

## Checklist failures

- R7: `search` (lines 62-80) mutates six outer `var`s and restores `taken` and `tops` by hand; reported (medium) with a stdlib fix.
- R6: `search(p: Int)` (line 62) carries a pizza as a raw `Int` and rebuilds it with `Pizza(p)` (lines 69, 75), ending at `p == n` (line 64).
- R4, minor: `surplus: [Flavour]-1e6..=1e6` and `stack: [Flavour]0..=1e6` (lines 29, 41) restate the global limit instead of `-n..=n` and `0..=n`.
- Stale friction: "Domain as a permutation target" describes `a.is_permutation(Flavour.collect())`, which the port no longer uses (line 31 is `surplus.all(c => c <= 1)`).

## Owed rewrites

- Recurse over `?Pizza` (`p.next`) instead of a raw `Int`.
- Bound `surplus` and `stack` by `n`.
- Drop or update the stale `is_permutation` friction.

## Language gaps

- A pruning `matchings` generator and `decreasing_piles` returning each element's pile (lines 51-80, and the hand patience piles at lines 40-47): in the friction, with a run.
- Whole-input buffering costs 1.14x the C++'s memory: in the friction (codegen).

## Dream notes

- A fragment over two branches, judged as the port with them. `matchings(...).map(decreasing_piles).min(by:, prune:)` names the objects; the composite still carries the raw-`Int` bounds above, and the distinct branch should also use `decreasing_piles` rather than lines 40-47.
