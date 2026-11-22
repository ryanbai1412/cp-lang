// `<<` in an array type's count, range and range-type bounds.
let n = 3
var dp: [0..1 << n]Int = 0
var half: [1 << (n - 1)]Int = 1
let small: List<0..1 << 3> = [7]
print(dp.len, half.len, small[0])
