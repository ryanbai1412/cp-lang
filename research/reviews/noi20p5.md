# noi20p5

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The spine words in a std `Trie<Shape>` and the all-four-children check follow the header.

## Checklist failures

- Line 20 spells the case marker `@multitest` (idiomatic-code/R8).
- Lines 35-54 walk with `at = None` reset each step and an `if/elif` chain on `l == None`, `r == None`, `leaf(l!)`; a `match (l, r)` over the four shapes says it without unwraps.
- Lines 57-65 search for a failing trie node with a `todo` stack and an `ok` flag (idiomatic-code/R3: `any`/`reachable`).

## Owed rewrites

- Write the dream.
- `print(if reachable(...).any(o => ...)...)` or an `all` over the trie's reachable nodes in place of the flag loop.

## Language gaps

- None.
