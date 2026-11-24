// The edits of a SortedBag of n keys: each removes what it added, so the size stays n.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
var b = SortedBag<Int>.of(xs)
let atleast = (x: Int) => x >= n / 2 // within: bag.SortedBag.take_first
for i in 0..8 * n:
    let k = i * 13 % n
    b.add_many(k, 3) // timed: bag.SortedBag.add_many
    s += b.remove_all(k) // timed: bag.SortedBag.remove_all
    b.add(k)
    let c = b.cursor_at(k)!
    b.remove_at(c) // timed: bag.SortedBag.remove_at
    b.add(k)
    let f = b.pop_first()! // timed: bag.SortedBag.pop_first
    b.add(f)
    let l = b.pop_last()! // timed: bag.SortedBag.pop_last
    b.add(l)
    let t = b.take_first(atleast)! // timed: bag.SortedBag.take_first
    b.add(t)
    let got: List<Int> = b.drain_range(k..k + 4).collect() // timed: bag.SortedBag.drain_range [fixed: k]
    for x in got: b.add(x)
for i in 0..64 * n:
    var e = SortedBag<Int>() // timed: bag.SortedBag.new
    e.add(i)
    let t = e.take() // timed: bag.SortedBag.take
    s += t.len
    e.add(i)
    e.clear() // timed: bag.SortedBag.clear
    s += e.len
let ys = [i * 31 % n + n / 2 for i in 0..n]
for r in 0..16:
    var big = SortedBag<Int>.of(xs)
    let d = big.drain(after: n / 4, while: x => x < n / 2) // timed: bag.SortedBag.drain
    s += d.len
    var hi = big.split_at(n / 2) // timed: bag.SortedBag.split_at
    big.join(hi) // timed: bag.SortedBag.join
    var o = SortedBag<Int>.of(ys)
    big.absorb(mut o) // timed: bag.SortedBag.absorb
    s += big.len
print(s)
