# Scout batch 26: hard trees

Ten tree submissions from the corpus, re-solved in CP Lang: centroids,
heavy-light paths, subtree ranges, Euler order, binary lifting, a
reachability (Kruskal) tree, a mergeable DP, an auxiliary (virtual) tree
and one interactive problem. This is a Stage-1 critique pass. Nothing in
`proto/cpc/`, `crates/` or `stdlib/` was changed for it. What the ports
taught us is in
[`research/archive/frictions/friction-backlog-batch-26-trees.md`](../../../research/archive/frictions/friction-backlog-batch-26-trees.md).

Each port was written from the statement first (DMOJ API metadata plus the
published contest text), then the original was read for the exact input
and output format. Every port runs today on the reference interpreter and
`check.py` compares each one with its original on small random trees:

```
$ python3 proto/ports/scout_batch_26/check.py              # every port, 60 inputs each
$ python3 proto/ports/scout_batch_26/check.py noi20p2 300  # one port, 300 inputs
```

The inputs are small (at most 14 vertices; the sizes are in `check.py`).
No port was run at the problem's largest size or timed, and none was
built with `proto/cpc`. Two ports would not pass at full size as written,
and say so in a `WISH` comment: `noi20p2` keeps dense DP rows (O(n^2) on a
path) where the accepted solution merges sparse segment trees, and
`noip19p6` recurses to the depth of the tree.

## The ports

The line counts leave out blank lines and comments; the original's count
includes its template.

- `noip19p6` (NOIP '19 P6, Centroid of Tree; original 5148136, 215 lines,
  port 49). Root at the centroid; a vertex is a centroid of its part when
  the size cut away falls in a range, counted with Fenwick trees during
  one walk. The original merges segment trees instead.
- `ioi09p7` (IOI '09 P7, Regions; 5032754, 118 → 52). Square-root split on
  region size: one preorder sweep per big region, binary search over
  subtree ranges for small ones. Answered online.
- `dmopc20c7p4` (DMOPC '20 C7 P4, Mimi and Carrots II; 3686586, 269 → 25).
  Safe vertices on or next to a path: one `Hld` path sum over "safe
  children" plus two lookups at the top.
- `mmcc14p3` (MMCC '14 P3, Esdeath; 5120083, 89 → 34). k centres with the
  least radius: bisection over a bottom-up greedy.
- `btoi15p3` (Baltic OI '15 P3, Network; 5640865, 109 → 17). Join leaf i to
  leaf i + k/2 in preorder. Any optimal answer is accepted, so `check.py`
  checks the count and that no bridge is left.
- `noip19p2` (NOIP '19 P2, Brackets on a Tree; 5127828, 186 → 31). One
  top-down pass with a "nearest open bracket above" pointer; the original
  keeps a segment tree with rollback.
- `sdoi17r1p2` (SDOI '17 R1 P2, Tree Colouring; 5463990, 376 → 43). Paint
  root paths, count colours on a path, largest count in a subtree. The
  link-cut "access" done over heavy chains with a lazy tree on preorder
  positions.
- `noi20p2` (NOI '20 P2, Destiny; 5655455, 335 → 35). The depth-indexed DP
  with the heavy child's row reused. Dense rows; see above.
- `ioi18p3` (IOI '18 P3, Werewolf; 4727179, 201 → 56). Two reachability
  trees, a climb in each, then a rectangle-emptiness sweep. The original is
  the grader interface, so `check.py` appends a stdin/stdout `main`.
- `ceoi22p3` (CEOI '22 P3, Prize; 4728198, 204 → 50). Interactive: ask about
  neighbours in the second tree's preorder (the auxiliary tree of the
  picked set), then solve for depths. `check.py` plays the judge with
  hidden edge lengths and scores both programs against the true distances.

The corpus has no unported centroid-*decomposition* submission (`ioi11p2`
is already in `proto/ports/`), so the centroid entry here is the centroid
counting problem; the backlog covers decomposition from the shape that
`noip19p6` and `ioi09p7` would have needed.

## What the interpreter does not have

These came up while making the ports run and are worked around in place,
each next to a `WISH` comment: `bsearch_first`, `flush()`, printing a list
on one line, `while let x = stack.pop()`, reading a `[V]Char`, a domain
alias (`domain Pre = Rooted<V>.Pre`), an inline `[domain Q = 0..t]`, an
`if let` in an expression-bodied function, and `V.first + 1..` when the
domain has one element.

## Overlap with other batches

Three of these problems were also ported by batches that landed while this
one was being written: `noip19p2` (`blind_critic_3`), `noi20p2`
(`scout_batch_24`) and `ioi18p3` (`scout_batch_27`). The ports here were
written independently from the statements and are kept as a second
opinion: `noi20p2` in particular is here for the row-ownership question in
section 3 of the backlog, and `ioi18p3` for the predicate climb and the
two-tree `Pre` mix-up. The corpus has few unported tree problems left.
