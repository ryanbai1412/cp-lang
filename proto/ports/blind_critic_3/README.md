# Blind critic 3: the one-page cheat sheet under contest pressure

Eight corpus problems with no earlier port, each written first from
`docs/one-page-cheat-sheet.md` alone (`*_draft.cp`, kept as written; seven of the eight do not
build on `proto/cpc`), then turned into a port that builds on the prototype and agrees with the
original submission (`<problem>.cp`). The findings are in
[`research/blind-critic-3-cheatsheet-test.md`](../../../research/blind-critic-3-cheatsheet-test.md).

- `noip19p2` (tree + bracket matching, originally a segment tree over the root path): rethought as a stack of open brackets along the DFS path. Original 5127828.
- `coci21c2p5` (compression + range-add/max tree + two pointers + binary lifting). Original 4143748.
- `aac5p5` (sweep over horizontal levels + two Fenwick trees over compressed x). Original 4240171.
- `cco16p1` (DSU over paths and cycles + counting). Original 4583882.
- `cheerio1s4` (implicit graph over next/previous occurrences + Dijkstra). Original 5339437.
- `ioi09p6` (Mecho: multi-source grid BFS + binary search on the answer). Original 5106126.
- `cmt21r1xym` (sorting + counting argument + binomials mod p). Original 3997206.
- `noip22p1` (grid run lengths + running sums mod p, multi-test). Original 5213639.

`check.py` builds scratch copies (so the ports are never stamped) and runs each port against
its original on random inputs:

```
python3 proto/ports/blind_critic_3/check.py              # all eight, 300 inputs each
python3 proto/ports/blind_critic_3/check.py aac5p5 --iters 1000
```
