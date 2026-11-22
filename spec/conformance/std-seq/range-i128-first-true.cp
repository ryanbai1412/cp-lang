// expect: ok, output "100000000000000000777 -5 -5"
// rule: std.seq.first_true
let big: I128 = I128(1000000000000000000) * I128(100)
let a = (big..=big + I128(1000)).first_true(x => x >= big + I128(777))!
let lo = I128(-4_000_000_000) * I128(10_000_000_000)
let b = (lo..=I128(0)).first_true(x => x >= I128(-5))!
let c = (lo..=I128(0)).last_true(x => x <= I128(-5))!
print(a, b, c)
