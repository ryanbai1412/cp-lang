# Friction Report: noi21p1

No friction left. An edge is heavy exactly when both its ends were last painted by the same
update, so one std lazy `SegTree` over the heavy-light layout holds colour runs: a `?Run` of a
range's end colours and its count of equal neighbours (`None` is the empty range, not a `0`
sentinel), and the paint is an `Action<?Run, ?Int>` whose identity is `None`. An update paints
each `tree.chains(a, b)` range; a query folds them and compares colours across each range's top
edge via `tree.at`. The case loop is `@!multitest`, and every read is typed.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's cases pass. On the max case (T = 3, n = m = 1e5) the port takes 0.48 s
and the C++ 0.52 s, with the same output.
