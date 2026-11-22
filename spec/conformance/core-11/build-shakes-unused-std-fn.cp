// expect: ok
// rule: 11.2.4
// tool: build -o -
// has: "_zip_"
// lacks: "_merge_sorted_"
use std.seq.{zip, merge_sorted}
for (a, b) in zip([1, 2], [3, 4]):
    print(a + b)
