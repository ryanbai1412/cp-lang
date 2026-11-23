// expect: E0203 at 4
// has: "empty when its `len` is 0"
// On a type spelled with `len`, the unknown-member error for `is_empty` offers `len == 0` (bug-catch round 14, 28ca9a32).
domain B = 0..4
let s = Subset<B>.empty
if not s.is_empty: print(1)
