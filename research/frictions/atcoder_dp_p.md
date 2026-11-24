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
accepted C++ carries, and the port beats it on time and memory. The header has no
`original:` line although meta.json names the reference; that is a header edit owed
by idioms, tracked for the lint in the atcoder_dp_g report (Owner: Consumers).
