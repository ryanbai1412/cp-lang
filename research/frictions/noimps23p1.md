# Friction Report: noimps23p1

No friction left. The v0.2 port's untyped reads, the 0-based shift, the hand-written
path-compressed `nxt` skip pointers, the `seen` array and the per-call `starts_at` lists are gone.
Stations and tracks are domains with typed reads, the swept stations are a std `IntervalSet`
whose `next_gap` finds the next unswept station, the tracks starting at a station are
`line.map(.l).buckets()`, and the answer prints as one list.

Evidence: stress agrees on 30/30 cases against the v0.2 port and 30/30 against the accepted
C++, and the package's cases pass. On the max case the port takes 0.04 s and the accepted C++
0.14 s, with byte-identical output.
