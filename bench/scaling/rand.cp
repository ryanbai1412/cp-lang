use std.rand.{Rng}
let n: Int = read()
domain D = 0..n
var r = Rng(7)
var acc = 0
let xs = [i for i in 0..n]
var a = [i for i in 0..n]
for i in 0..8 * n:
    var q = Rng(i) // timed: rand.Rng.new
    acc ^= Int(q.next() >> 4) // timed: rand.Rng.next
    acc += r.int(0, i) // timed: rand.Rng.int
    if r.float() < 0.5: acc += 1 // timed: rand.Rng.float
    if r.chance(0.25): acc += 1 // timed: rand.Rng.chance
    acc += r.pick(xs) ?? 0 // timed: rand.Rng.pick
for _ in 0..48:
    r.shuffle(mut a) // timed: rand.Rng.shuffle
    let p = r.perm<D>() // timed: rand.Rng.perm
    acc += p[D(0)].ord
    acc += r.sample(n / 2, from: xs).len // timed: rand.Rng.sample
print(acc, a[0])
