# Friction Report: noi20p2

No friction left: the simulated node pool (parallel `lc`/`rc`/`sum`/`mul` lists with node 0 as
a null sentinel, a reserve guess, a hand-pushed multiply tag and recursive set/sum/merge over
`lo`/`hi`) is std `MergeTrees<Dep, Z>`. Each vertex's DP is a `MergeTree<Dep>` handle, the
child's freed states are `pool.sum` over a `Dep` range, and the max-convolution is
`pool.merge` with a hook over both values and both prefix sums, the child side pre-scaled by
the freed count. Values are `Mod<998_244_353>` instead of `% mod`, the top depth of a pair
is the domain `Dep`, and every read is typed.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++. On the max case the port takes 1.08 s and the C++ 0.59 s, with the same output, inside
the 2 s limit.
