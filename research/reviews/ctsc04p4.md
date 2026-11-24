# ctsc04p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A tree knapsack over a rooted network. `plan(u)` returns, for each server v the subtree could route to, a staircase of (cores, cost) steps; children merge largest-first into `best: [0..=extra]?Int`.

## Checklist failures

- The merge is a hand-written 0/1 knapsack (`for j in (0..=min(extra, held)).rev`, then `best[j] = None` and `min=` into `best[j + s.cores]`) inside a per-v comprehension.
- `staircase` rebuilds the Pareto steps with a `var` list and `return steps`, a hand filter that keeps only cost changes.
- `plan` recurses over children and materialises `kid_plans` as a list of tuples; a bottom-up pass over `tree.at.rev` would avoid recursion.
- `extra = min(p - 1, n)` caps the core count by hand without naming why n is enough.

## Gaps that need the language

- A generator can't draw from an optional: research/frictions/ctsc04p4.md.
