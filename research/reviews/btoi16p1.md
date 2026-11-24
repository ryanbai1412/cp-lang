# btoi16p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction)

Cold review of `port.cp`, without the C++. The boss -> report graph is `Graph.of` and each root is
priced with one std `bfs`. `payroll(root) -> ?Int` is None when the root misses someone, and the
answer is the `min` over the roots that reach everyone. With no dream the port scores at most 3.

## Checklist failures
- idiomatic-code/R1: the comment on line 17, "everyone's best pay is their depth counting the
  root as 1", is false per person. A boss's least salary is their subtree size, and only the
  total equals the sum of depths (each employee counted once per ancestor). The header's
  "1 + reports" also reads as a count, not as "1 + the reports' salaries".

## Owed rewrites
- Fix the header and line 17: salary = subtree size, so the total is the depth sum.
- Write a dream (the port is close to one).

## Language gaps
None. The friction measures a closure-`bfs` draft at 0.56 s against 0.11 s for `Graph.of`.
