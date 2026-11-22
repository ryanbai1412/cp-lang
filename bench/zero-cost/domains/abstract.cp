fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    domain V = 0..n
    var arr: [V]Int = 0
    for i in V:
        arr[i] = a[i.to_idx]
    var acc = 0
    for _ in 0..reps:
        for i in V:
            acc = (acc + arr[i] * (i.to_idx + 1)) ^ (acc >> 3)
    acc

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
