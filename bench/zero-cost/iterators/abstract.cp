fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var total = 0
    for r in 0..reps:
        total += a.filter(x => ((x + r) & 1) == 0).map(x => (x + r) * 3 + 1).sum()
    total

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
