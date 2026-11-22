use std.rand.{Rng}
print('7'.digit, zip([1, 2, 3], [4, 5]))
for w in [1, 2, 3, 4].windows(2): print(w)
print([w for w in [1, 2].windows(3)].len)
var r = Rng.new(5)
print(r.int(1, 6), r.int(1, 6), r.next())
domain R = 0..3
domain C = 0..3
let p = (R(1), C(1))
let q = p.step(Dir8(1))!
print([p.step(d) for d in Dir8.all()].len, q.0.ord, q.1.ord, find_all("abab", "ab"))
