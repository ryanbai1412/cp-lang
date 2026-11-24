# Friction Report: noi17p1

No friction left: the unused subtask hints are read into `_`, digits are typed `0..BASE`, and the
-1 sentinel max SegTree of differing digits became an or-SegTree searched with `left_while`, so
"no lower digit differs" is a cut at 0 rather than a magic value. The carry loop is one `add`.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 1e6) 0.21 s vs C++ 0.59 s, same
output.
