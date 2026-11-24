# btoi17p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction)

Cold review of `port.cp`, without the C++. The header derives the degeneracy bound (a least-degree
vertex has < K remaining neighbours). The port removes vertices in that order with a lazy
`MinHeap` and fills a `[Near.subsets]Bool` clique table over a local `Near` domain, using the
recurrence "first member knows the rest and the rest is a clique". With no dream the port
scores at most 3.

## Checklist failures
- idiomatic-code/R6: `near[i.ord]` and `near[j.ord]` (line 37) index a `List<P>` by a `Near`'s
  ordinal. A `[Near]P` table (`[Near] i => ...`) would keep the domain.
- The lazy-deletion pop (`if gone[u] or d != deg[u]: continue`, line 27) is the C++ idiom for a
  degeneracy order; fine, but a dream should say whether std should own it.

## Owed rewrites
- Write a dream.

## Language gaps
None; the friction reports none.
