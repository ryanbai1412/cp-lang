# cco17p3

Score: 3/5
Dream: 3/5

The residue-class argument is in the header and the program follows it: an x trie of low bits, each node owning a `Column` (a y trie and its colour sums), and a query that walks both. The node payloads are kept beside the tries by hand, the walk is written twice, and it runs 1.85x slower than the C++.

## Checklist failures

- idiomatic-code/R4: the "follow x's low bits until the trie stops" loop is written twice, in `walk` (lines 21-26) and `colour_along` (lines 33-41).
- idiomatic-code/R6: `column_at: HashMap<Int, Int>` and `colour: List<Int>` index trie nodes and columns by raw `Int` (reported: no per-node storage).
- idiomatic-code/R10: 1.33 s against 0.72 s at `--max` (1.85x), and the friction had no performance section (added).

## Owed rewrites

- `colour_along(col, c) = sum(col.colour[p] for p in walk(col.ys, c))` reuses `walk`; measure it, since it allocates the path.

## Language gaps

- A trie cannot carry a value per node; walking a key down a trie is by hand: research/frictions/cco17p3.md.
- The 1.85x slowdown: research/frictions/cco17p3.md, "The port is 1.85x slower than the C++" (added by this review).

## Dream notes

- `xs[node].get_or_insert(Trie.new())[xs[node]!.add(low_bits(s.y))] += s.v` inserts into two tries inside one index expression (idiomatic-code/R7, caps at 3). Bind the column and the y node in their own statements, then add `s.v`.
