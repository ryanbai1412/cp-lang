# Scout batch 10: strings

Ten string-algorithm submissions from `corpus/submissions/`, rethought in
CP Lang for a Stage-1 friction pass. None of them is in `proto/ports/` or an
earlier scout batch. The friction they showed is in
[`research/friction-backlog-batch-10.md`](../../../research/friction-backlog-batch-10.md)
(items `T1`...).

Every port here runs today on the reference interpreter and is checked
against its original:

```sh
python3 proto/ports/scout_batch_10/check.py               # every port, 60 random inputs each
python3 proto/ports/scout_batch_10/check.py noi15p5 200   # one port, 200 inputs
python3 proto/cp.py test proto/ports/scout_batch_10       # the tests of hashing.cp
python3 proto/cp.py lib-run proto/ports/scout_batch_10/saspeed.cp < input
```

The inputs are small (strings of up to 16 letters, built to repeat), so the
check is about answers, not speed: the interpreter is far too slow for the
real limits, and two ports are known to be too slow at them even compiled
(saspeed, T6; nccc10s3, T8).

The ports, with their non-blank, non-comment lines against the original's
non-blank lines:

- `ctsc06p1.cp` (15 / 128): KMP borders. The expected wait for a word is a
  sum over its borders, which replaces the original's recurrence.
- `btoi19p5.cp` (24 / 204): KMP. The longest rotated or flipped common
  block, by running the failure function from every cut; the original
  binary-searches hashes.
- `noip20p2.cp` (28 / 163): Z-function in place of the original's hash, and
  a 27-entry table in place of its Fenwick tree.
- `saspeed.cp` (3 / 105): suffix array, straight from `std.strings`.
- `noi15p5.cp` (30 / 285): suffix array, LCP array and union-find.
- `nccc10s3.cp` (18 / 203): suffix array and two binary searches for the
  k-th occurrence of a pattern.
- `noi16p1.cp` (22 / 138): rolling hash, longest common prefix and suffix
  of two places.
- `noi17p2.cp` (46 / 316): rolling hashes joined end to end, counted in a
  map.
- `year2018p7.cp` (29 / 159): rolling hash, sets merged small to large up a
  tree, binary search on the answer.
- `sac22cc3p5.cp` (30 / 108): a trie where the original hand-builds a hash
  table.

`hashing.cp` is not a port. It is the hashing module the three hash ports
wanted from the standard library (T1), written in CP Lang with its own
tests, and imported as `use scout.hashing.{Hash, StrHash}` through
`lib.toml`. It is here, not in `stdlib/`, because a scout batch proposes
and a Stage-2 sprint decides.

The corpus has no accepted, unported submission that uses Aho-Corasick or
Manacher (searched for failure links, suffix links and mirrored radii), so
neither is in the batch. sac22cc3p5 is the problem Aho-Corasick would
solve at larger limits, and T10 says what a library trie would need to
carry it.
