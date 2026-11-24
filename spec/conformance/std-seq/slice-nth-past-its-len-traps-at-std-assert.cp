// expect: R0401
// rule: std.seq.Slice.nth
// A slice's `nth(k)` past its length stops at std's `Slice.nth` assert on both engines, the built-in view as much as std's struct.
let xs = [3, 1, 4, 1, 5]
print(xs[1..3].nth(5)) // <- here
