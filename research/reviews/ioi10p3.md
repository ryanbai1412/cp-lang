# ioi10p3

Score: 4/5
Dream: 4/5 (the friction: the port is the dream)

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 4/5. Binary search on the answer with `first_true`, and a 2D prefix count of `rank <= x` tested over every window with `windows` (lines 16-22): the header in seven lines.
- Lines 18-20 fill the `Prefix` cell by cell with `.filling()` and `set`; the constructor from a table, `Prefix<Row, Col, Int>([Row, Col] (i, j) => Int(rank[i, j] <= x))`, says it in one line.
