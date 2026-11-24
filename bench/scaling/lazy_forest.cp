// sizes stay at n: each split is paired with an append
use std.lazy.{LazyForest}
let n: Int = read()
let reps = 1 << 17
var s = 0

fn forest() -> LazyForest<Int, Int> = LazyForest<Int, Int>(apply: (v, k, g) => v + g, compose: (g, h) => g + h)
for i in 0..reps:
    var e = forest() // timed: lazy.LazyForest.new
    let q = e.seq()
    s += e.len(q) + i % 2
var fo = forest()
let fs = fo.seq()
for i in 0..n: fo.push(fs, i * 7919 % n)
for i in 0..reps:
    let p = i * 7919 % n
    s += fo.len(fs) // timed: lazy.LazyForest.len
    fo.apply(fs, p / 2..p, 1) // timed: lazy.LazyForest.apply
    fo.reverse(fs, p / 3..p) // timed: lazy.LazyForest.reverse
    s += fo.get(fs, i * 13 % n) // timed: lazy.LazyForest.get
    let u = fo.split_at(fs, p) // timed: lazy.LazyForest.split_at
    fo.append(fs, u) // timed: lazy.LazyForest.append
    let e = fo.seq() // timed: lazy.LazyForest.seq
    fo.push(fs, i) // timed: lazy.LazyForest.push
    fo.append(e, fo.split_at(fs, n))
for _ in 0..200:
    s += fo.to_list(fs).len // timed: lazy.LazyForest.to_list

print(s)
