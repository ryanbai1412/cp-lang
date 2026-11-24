# sac22cc2p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: the forks read as one typed table over a Fork domain into a Node domain; parents and fork owners are optional tables, Rooted.from_parents orders the tree, and Rooted.up does the DP with lift adding the gain only on the take edge. No BFS queue, edge lists or sentinel -inf by hand.

## Checklist failures

- The fork table is rebuilt into `parent` and `fork_at` arrays only so that `Rooted.from_parents` can run the tree DP.
- `lift` tests `if let f = fork_at[p] and forks[f].take == c` inside the fold, which mixes the edge's meaning (take or skip) into a closure over parents.
- At the `--max` case the port ran 0.19 s against the accepted C++'s 0.05 s, both far under the limit; the gap is the rebuild plus `t.up`'s generic fold.

## Gaps that need the language

- The fork table rebuilt into a parent array: research/frictions/sac22cc2p3.md.
