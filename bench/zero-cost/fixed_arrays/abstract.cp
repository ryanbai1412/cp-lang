domain K = 0..4

fn bump(c: [K]Int, x: Int) -> [K]Int = [K] k => (c[k] * 31 + x + k.ord) % 1_000_000_007

fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var c: [K]Int = 0
    for _ in 0..reps:
        for i in 0..n:
            c = bump(c, a[i])
    c[K(0)] + c[K(1)] + c[K(2)] + c[K(3)]

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
