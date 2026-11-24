fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var c0 = 0
    var c1 = 0
    var c2 = 0
    var c3 = 0
    for _ in 0..reps:
        for i in 0..n:
            let x = a[i]
            c0 = (c0 * 31 + x) % 1_000_000_007
            c1 = (c1 * 31 + x + 1) % 1_000_000_007
            c2 = (c2 * 31 + x + 2) % 1_000_000_007
            c3 = (c3 * 31 + x + 3) % 1_000_000_007
    c0 + c1 + c2 + c3

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
