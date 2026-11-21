use std.prefix.{prefix1}

fn first_big<D>(a: [D]Int, x: Int) -> ?D:
    for i in D:
        if a[i] >= x: return some(i)
    none

fn doubled<D>(a: [D]Int) -> [D]Int = [D] i => 2 * a[i]

fn total<D>(a: [D]Int) -> Int = sum(a[i] for i in D)

let n: Int = read()
let a: [0..n]Int = [0..n] i => i * i
let p = prefix1(a)
print(p.prefix(0), p.prefix(2), p.prefix(n), p.sum(1..3), p.sum(1..=3), p.sum(2..2))
let b: [1..=n]Int = [1..=n] i => 10 * i
let q = prefix1(b)
print(q.sum(1..=n), q.sum(2..4), q.prefix(1), q.prefix(n + 1))
print(first_big(a, 5) ?? -1, first_big(b, 25) ?? -1, first_big(a, 100) ?? -1)
let d = doubled(b)
print(d, d[1], d[n], total(a), total(b), total(doubled(a)))
print(p.sum(4..9))
