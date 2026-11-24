# dmopc21c7p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A doubly linked list over colour spots: each op cuts l..=r out and splices it after k, then the list is walked from the front.

## Checklist failures

- `next` and `prev` are parallel arrays kept consistent by `link` by hand; a linked-list type over the domain would own the splice.
- `prev` uses `s.prev ?? s` so the front points at itself as a sentinel.
- `scarf` is collected by `while let s = at` with `var at`, a hand iterator over `next`.
