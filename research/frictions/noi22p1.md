# Friction Report: noi22p1

No friction left: the back-linked node lists with `-1` links and head/tail/size arrays are a
std `Deque` per store, and the one shared HashMap keyed by `store << 20 | value` is a std
`Counter` per store, merged small to large with `absorb`. Stores, sequence labels and bits are
domains, every read is typed, and the query's per-bit majority and the candidate's copies are
`sum` over the picked stores. A join moves the shorter deque onto the right end of the longer
one (`push` for a shorter back half, `push_front` in reverse for a shorter front half).

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++. On the max case the port takes 0.68 s and the C++ 0.91 s, with the same output.
