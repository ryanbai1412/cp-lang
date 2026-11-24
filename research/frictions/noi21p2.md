# Friction Report: noi21p2

No friction left: the hand-rolled `List<List<Z>>` path counts and Gaussian elimination are a
std `Mat<Z>` (a row per vertex of the current layer, a column per start), built with
`identity`, `zeros` and `at`, and std `det`. Each edge u -> v adds u's row to v's, so a layer
costs O(n_1 m_j) instead of a dense product. The layer sizes, edge counts and endpoints are
typed reads, and the case loop is `@multitest`. Every closure inlines (`cpl explain`: 2 static,
0 dynamic).

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, on the package generator. On its max case (T = 5 tests of 100 layers of 200 vertices) the
port takes 1.28 s and the accepted C++ 2.60 s, with the same output. tests.toml leaves out
`--max` because the accepted C++ runs it past the 2 s limit.
