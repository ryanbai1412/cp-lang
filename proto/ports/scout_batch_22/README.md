# Scout batch 22: hard strings, automata and parsing

Ten string, automaton and parsing submissions from `corpus/submissions/`,
rethought in CP Lang. None is in `proto/ports/`, an earlier
scout batch or a `blind_critic_*` directory. The friction they showed is in
[`research/friction-backlog-batch-22-strings.md`](../../../research/friction-backlog-batch-22-strings.md)
(items `X1`...).

Batch 10 took the corpus's accepted KMP, Z-function, suffix-array and
rolling-hash submissions, and the corpus has no accepted Aho-Corasick or
Manacher one. This batch is what was left that is hard in a string-shaped
way: a suffix automaton, a trie, a shortest path and a search whose states
are strings, two string DPs, and three kinds of parsing. Two more were
written and then dropped, because batch 19 landed with them while this
batch was in progress: ceoi22p2 and noimps22p1 (a preprocessor, which
needs `read_line()`; see X1 in the backlog).

```sh
python3 proto/ports/scout_batch_22/check.py               # every port, 60 random inputs each
python3 proto/ports/scout_batch_22/check.py noi18p3 200   # one port, 200 inputs
python3 proto/cp.py test proto/ports/scout_batch_22       # the test of bits.cp
python3 proto/cp.py lib-run proto/ports/scout_batch_22/ioi08p1.cp < input
```

Every port agrees with its original on 60 small random inputs
(`check.py`, seed 22). The inputs are small (strings of up to a dozen
letters), so the check is about answers, not speed; nothing was run at the
real limits. Four problems accept many answers (ioi08p1, noi09p2, ccc01s5,
ccc99s5), and there both outputs are judged against the statement. Two
originals are IOI functions without a `main` (ioi16p4, ioi21p4); `check.py`
appends one that reads the format in the port's header.

The ports, with their non-blank, non-comment lines against the original's
non-blank lines:

- `noi18p3.cp` (91 / 537): a suffix automaton whose states carry
  persistent mergeable segment trees of end positions, and a second
  automaton per query.
- `ioi08p1.cp` (42 / 87): a trie walked so that the longest word comes
  last.
- `cco19p6.cp` (30 / 97): Dijkstra where a vertex is a `Str`, the part by
  which one decoding is ahead of the other.
- `ccc01s5.cp` (22 / 72): Post's correspondence problem, a depth-first
  search over the same kind of overhang.
- `ioi16p4.cp` (50 / 125): a string DP run on the row and on the row
  reversed.
- `ioi21p4.cp` (26 / 88): prefix counts of letter pairs.
- `noi09p2.cp` (42 / 244): typesetting a poem, a convex-cost DP with
  monotone breaks; sentences are joined back into lines for the output.
- `ccc99s5.cp` (36 / 98): a cryptarithm solved column by column; the
  original tries all 10! assignments.
- `noi14p1.cp` (16 / 102): `AND 5`-style tokens and bit tricks.
- `noi21p4.cp` (37 / 121, plus `bits.cp`): 64 hex digits
  per query, a generator given as C code on wrapping 64-bit words, and a
  pigeonhole on 16-bit blocks.

`bits.cp` is a `U64` made of two halves, standing in for the core type of
the same name (X9); `lib.toml` lets the interpreter resolve
`use scout.bits`.

How statement-first this was: `dmoj.ca/api/v2/problem/<code>` answered 403
(a Cloudflare challenge) for every code in this batch from the machine
that did the work, so no metadata came from it. The ports were written
from the published IOI, CEOI, CCO, CCC and NOI statements as known. Before
writing, part of seven originals was read for the input and output format
(ioi08p1, cco19p6, ccc01s5, ccc99s5, noi21p4, noi09p2 and the last lines
of ioi16p4), which for cco19p6 also shows its algorithm, and
noi21p4's generator was copied from its original because the statement
gives it as code. noi18p3, noi14p1 and ioi21p4 were written without
opening theirs.
