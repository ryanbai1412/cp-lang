# atcoder_dp_p: friction report

Port: `corpus/problems/atcoder_dp_p/port.cp`, dream: `corpus/problems/atcoder_dp_p/dream.cp`.

## Measurements

- Tokens: port 151, accepted C++ 451.
- Emitted C++: 23448 bytes.
- `--max` (n = 1e5, median of 15): port 11.3 ms / 11.7 MB, accepted 20.1 ms / 17.9 MB (0.56x time).
- Output: identical to the accepted C++.

## Dream

The port is the dream: the code below the header is token-for-token `dream.cp`. Two
tables `white` and `black` filled children-first over `tree.at.rev`, with
`tree.parent(v)` as an option, is how the editorial states the recurrence.

## Frictions

None. The `Rooted` traversal removes the recursive DFS and the parent sentinel the
accepted C++ carries, and the port beats it on time and memory.

## Addendum: the root named once, and the reference's stack
- **Where:** the old answer line, `print(white[V.first!] + black[V.first!])`, restated the root
  that `Rooted(..., V.first!)` already fixed.
- **Fix (this port and its dream):** `let root = tree.root` / `print(white[root] + black[root])`.
  `Rooted.root` was already in std, so no std change was owed. `corpus.py verify` gives 0 findings.
- **Reference stack:** `corpus/submissions/4131860.cpp` runs a recursive DFS n deep. At `--max`
  (a path of 1e5 vertices) it segfaults under the default 8 MB stack. `corpus.py`
  (`judge_stack`) and `cpl test --make` (`crates/cpl-contest/src/package.rs`, `judge_stack`)
  both run references under `ulimit -s unlimited`, so make and verify reproduce the judge, and
  no Tools gap is owed. atcoder.jp's rules pages don't state AtCoder's stack limit, so the
  accepted C++ stays as it was. The port has no recursion.
- **Measurements after the rewrite:** port 144 tokens, 23,300 bytes of C++; `--max` best of 3:
  port 0.01 s / 12.3 MB, accepted 0.01 s / 18.8 MB, same output.
