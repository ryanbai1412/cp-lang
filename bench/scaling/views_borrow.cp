// Views borrow their source: making one, reading its `.src` again and again and
// walking a short range of a SortedBag cost the same at n and at 4n.
let n: Int = read()
let ys = [i * 7919 % n for i in 0..n]
let odd_ = (x: Int) => x % 2 == 1 // within: seq.filter
var s = 0
for i in 0..16 * n:
    let f = ys.filter(odd_) // timed: seq.filter
    s += f.src.len // timed: seq.Filtered.src
    s += f.src.len + i // timed: seq.Filtered.src
    s += ys.items.len // timed: seq.items
let b = SortedBag<Int>.of(ys)
for i in 0..4 * n:
    s += b.range(i % n..i % n + 8).each().count() // timed: bag.SortedRange.each [fixed: k]
print(s)
