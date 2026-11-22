use std.ntt.{convolve_subset, convolve_xor, fwht}

type Z = Mod<998_244_353>

let a = [1, -2, 3, -4, 5, -6, 7, -8]
let b = [2, 1, 0, -1, 3, 2, -2, 4]
var w = [x for x in a]
fwht(mut w)
print(w)
fwht(mut w, inverse: true)
print(w == a)
print(convolve_xor(a, b))
print(convolve_subset(a, b))

let ma = [Z(i + 1) for i in 0..8]
let mb = [Z(i * 3 + 2) for i in 0..8]
var mw = [x for x in ma]
fwht(mut mw)
fwht(mut mw, inverse: true)
print(mw == ma)
print(convolve_xor(ma, mb))
print(convolve_subset(ma, mb))
