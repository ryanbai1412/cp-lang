# Friction report: coci25c3p3

Rethought. The v0.2 port imported `two_color` from a scout library that no longer exists, so its frozen snapshot only builds once that import is pointed at `std.graph`. It peeled through a `head` index over a growing list and tracked the best chain with a `best_len = -1` sentinel beside `start: ?V`. It built the answer line by string concatenation. The v0.3 port passes the graph itself to `std.graph`'s `two_color(g)` and peels with a `Deque`, filtering each neighbour walk with `for a in g.out(u) if ..`. It records the peel order and starts the answer at `peeled.argmax(by: chain)` (first maximum, as before), then prints the tellers as a list. n and m are typed.

No language gap.

## Measurements

gen.py --max (N = 5e5, M = 1e6): port 0.45-0.62 s under an 8 MB stack (v0.2 port 0.55-0.97 s on the same machine). The accepted C++ recurses and dies (signal 11) at 8 MB; with an unlimited stack it takes 1.45 s and gives the same output.
