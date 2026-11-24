# Friction Report: noi21p5

No friction left: the hand-rolled implicit treap (a `Node` record with `-1` child links, an LCG
priority, explicit `a00..a11` fields, recursive split/merge and push/pull of the flip and
reverse tags) is std `SeqFold`. It keeps each run's matrix product in both directions, so
REVERSE is `reverse(l - 1..r)` with no reliance on E being W transposed, and FLIP is an
`Action<M2, Bool>` that conjugates by the swap matrix. Values are `Mod<998_244_353>`, and the
matrix is a named tuple type with one `mul`.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++. On the max case the port takes 0.41 s and the C++ 0.35 s, with the same output.
