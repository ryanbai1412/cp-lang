fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    let P = 998244353
    var acc = 1
    for r in 0..reps:
        let shift = (r + 1) % P
        for i in 0..n:
            var v = acc * 31 % P + a[i] % P
            if v >= P: v -= P
            v += shift
            if v >= P: v -= P
            acc = v
    acc

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
