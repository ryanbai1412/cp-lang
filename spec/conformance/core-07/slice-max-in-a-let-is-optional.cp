// expect: ok, output "3 -1 1 -1"
// rule: 07.9.3
// A slice can be empty, so `max()`/`min()` of one is `?T` wherever the value goes, not only directly under `??`.
let xs: List<Int> = [3, 1, 4]
let m = xs[0..2].max()
let e = xs[0..0].max()
let s = xs[0..2]
let lo = s.min()
let none = xs[1..1].min()
print(m ?? -1, e ?? -1, lo ?? -1, none ?? -1)
