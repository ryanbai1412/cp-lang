fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var total = 0
    for r in 0..reps:
        var acc = r
        for i in 0..n:
            acc = (acc + a[i] * 7) ^ (acc >> 3)
        total ^= acc
    total

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
