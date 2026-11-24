# Friction Report: noi23p1

No friction left. The three stroke lists of `(line, lo, hi)` tuples, the hand-written merge of
touching strokes and the hand-written `first_ge` binary search are gone. Each kind's line keys
are compressed, every line holds a std `IntervalSet` (which merges overlapping and touching
runs and keeps `covered`), the sweep's Fenwick tree is indexed by the compressed vertical
lines with `first_ge` / `last_le`, and reads are typed by `domain Col` and `domain Row`.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's cases pass. On the max case the port takes 0.12 s and the accepted C++
0.14 s, with the same output.
