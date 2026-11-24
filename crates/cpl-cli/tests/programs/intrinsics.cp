fn ones(x: Int) -> Int = @intrinsic("popcount")
fn root(x: F64) -> F64 = @intrinsic("sqrt")

let n = 12
let bits = ones(n)
let low = ctz(n)
let high = clz(n)
let width = bit_width(n)
let up = bit_ceil(n + 1)
let neg = -7
let dist = neg.abs
let side = root(2.25)
let f = -2.5
let down = f.floor
let ceiling = ceil(f)
let words = popcount(U64(255))
print(bits, low, high, width, up, dist, side, down, ceiling, words)
