// expect: ok, output "18\n6 true 2\n2\n9"
// input: "2\n1 2\n3 4\n"
let n: Int = read()
var ps: List<(Int, Int)> = []
for i in 0..n:
    let (a, b): (Int, Int) = read()
    ps.push((a + i, b))
var s = 0
for (a, b) in ps:
    s += a * b
print(s)
let t = (1, (2, 3), true)
let (x, (y, z), ok) = t
print(x + y + z, ok, (t.1).0)
fn swap(p: (Int, Int)) -> (Int, Int):
    (p.1, p.0)
let (u, _) = swap(ps[0])
print(u)
let qs: List<(Int, Int)> = [(i, i * i) for i in 0..=3]
print(qs[3].1)
