fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var total = 0
    for r in 0..reps:
        var s = 0
        for i in 0..n:
            let x = a[i]
            if ((x + r) & 1) == 0:
                s += (x + r) * 3 + 1
        total += s
    total

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
