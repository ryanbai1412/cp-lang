# btoi17p4

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction)

Cold review of `port.cp`, without the C++. The header proves the degree bound, the bounded
search tree (at most p members, q cut pairs) and the submodular uncrossing. The port follows
that: `grow() -> ?List<V>` branches on the first undecided friend, `groups()` bails with
let-else, and the pair loop uses `V.after(i)`. With no dream the port scores at most 3.

## Checklist failures
- idiomatic-code/R7: `members` and `side` (lines 22-23) are globals that `group_of` resets
  (`members = [u]`, line 42) and `grow` mutates and undoes around its recursion. In the
  friction's `## Gap`.
- idiomatic-code/R3: `stamp`/`now`/`mark` (lines 48-52) are a hand membership table for
  `minus` and `valid`. In the friction (a `List` has no O(1) membership).

## Owed rewrites
- Nest `grow` inside `group_of` so the search state is local.
- Write a dream (groups as `Set<V>` with difference and boundary count).

## Language gaps
- No O(1) membership on a list-shaped group: in the friction.
