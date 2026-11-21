use std.geometry.{minkowski_sum}

let (na, nb): (Int, Int) = read()
var a: List<Pt> = []
for _ in 0..na:
    let (x, y): (Int, Int) = read()
    a.push(Pt(x, y))
var b: List<Pt> = []
for _ in 0..nb:
    let (x, y): (Int, Int) = read()
    b.push(Pt(x, y))
var checksum = 0
var last_len = 0
for _ in 0..20:
    let s = minkowski_sum(a, b)
    last_len = s.len
    for p in s: checksum += p.x + p.y
print(last_len, checksum)
