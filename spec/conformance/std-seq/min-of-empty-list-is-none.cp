// expect: ok, output "-1"
// `min` returns `?T`, `None` on an empty list, as `std/seq.cp` declares (round 24, B3).
var xs: List<Int> = []
let m: ?Int = xs.min()
print(m ?? -1)
