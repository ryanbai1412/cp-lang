fn step(x: Int, m: Int) -> ?Int:
    if (x & 3) == 0: return None
    Some((x * 31 + 7) % m)

fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var acc = 0
    for r in 0..reps:
        for i in 0..n:
            if let v = step(a[i] + r, 1000003):
                acc = (acc + v) ^ (acc >> 5)
            else:
                acc = acc ^ (i + 1)
    acc

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
