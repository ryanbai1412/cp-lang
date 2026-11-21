struct Pack(xs: List<Int>, ys: List<Int>)

fn pack(xs: List<Int>, ys: List<Int>) -> Pack:
    Pack(xs, ys)

fn twice(xs: List<Int>) -> Pack = Pack(xs, xs)

fn make(n: Int) -> Pack:
    let a = [i for i in 0..n]
    let b = [i * i for i in 0..n]
    return pack(a, b)

let n: 1..=100 = read()
let p = make(n)
let q = twice(p.xs)
print(p.xs.len + p.ys.len, q.xs.len, p.ys[n - 1])
