# ceoi17p2 friction

The old port carried two `WISH` comments and sorted with `[o.a for o in odds]`; the port
maps the columns with `odds.map(o => o.a).sorted(by: desc)`. The greedy itself is already
the dream's.

## Dream

The existing `dream.cp` is the target. Two gaps remain.

## Real-valued range types

- **Where:** `let odds: [n](a: F64, b: F64) = read()`. The statement bounds the odds to
  `1.0..=1000.0`, but a range type over `F64` does not exist, so the bound is not stated.
- **General fix:** closed `F64` ranges as types (checked on read like integer ranges).
- **Status:** open.

## Two lists merged by hand

- **Where:** the loop moves `i` and `j` by hand over two sorted lists. A merge of the two
  descending lists by "which side pays less" is not a fixed order (it depends on the running
  sums), so no std merge expresses it; the dream keeps the same loop.
- **Status:** not a gap in the language; noted for the reviewer.

Measured: `cpl stress` agrees with the old port and with the accepted C++ (5324109) on 30
cases each.
