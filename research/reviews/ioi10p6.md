# ioi10p6 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi10p6.md).

Reads like the editorial: `City`/`Road` domains, fans typed with the statement bound and the
2e9 total asserted, a std `Rooted` over `Graph.undirected`, subtree sums in reverse preorder,
and the answer as `City.argmin(by: busiest)!`. `busiest` states the two loads the header
derives (the parent side `total - below[c]`, each child's subtree).

## Checklist failures

- `below[City.first!]` stands in for the total (line 30); the total is a fact the port already
  asserts as `fans.sum()`, so naming it once (`let total = fans.sum()`) would say it directly (R4).
- `random_tree` is generator scaffolding the solution reads past.

## Language gaps

- Subtree sums as a hand loop (`Rooted.subtree_sums`) and the grader function task: both in
  research/frictions/ioi10p6.md.

## Dream notes

- At bar: `tree.subtree_sums(fans)`, `below.total` and one `argmin` line. The `@grader`
  signature is the judge's form, blocked by E0501 as reported.
