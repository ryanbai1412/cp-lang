// A `<<` in an array type's range does not parse; `let full = 1 << n` first works.
let n = 3
var dp: [0..1 << n]Int = 0
print(dp.len)
