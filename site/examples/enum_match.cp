enum Op:
    Add(Int)
    Sub(Int)

let n: 1..=100 = read()
var cur = 0
for _ in 0..n:
    let kind: 1..=2 = read()
    let x: Int = read()
    let op = if kind == 1: Op.Add(x) else: Op.Sub(x)
    match op:
        Op.Add(v) => cur += v
        Op.Sub(v) => cur -= v
print(cur)
