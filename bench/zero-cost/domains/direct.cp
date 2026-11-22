fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var arr = [0 for _ in 0..n]
    for i in 0..n:
        arr[i] = a[i]
    var acc = 0
    for _ in 0..reps:
        for i in 0..n:
            acc = (acc + arr[i] * (i + 1)) ^ (acc >> 3)
    acc

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
