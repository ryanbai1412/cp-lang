fn step(x: 0..=1e9, m: 1..=1e9) -> Int:
    if (x & 3) == 0: return -1
    (x * 31 + 7) % m

fn kernel(n: Int, reps: 0..=1000, a: List<0..10_000>) -> Int:
    var acc = 0
    for r in 0..reps:
        for i in 0..n:
            let v = step(a[i] + r, 1000003)
            if v != -1:
                acc = (acc + v) ^ (acc >> 5)
            else:
                acc = acc ^ (i + 1)
    acc

let n: Int = read()
let reps: 0..=1000 = read()
let a: [0..n]0..10_000 = read()
print(kernel(n, reps, a))
