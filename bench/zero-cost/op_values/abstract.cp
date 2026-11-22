fn fold_with(a: List<Int>, init: Int, op: (Int, Int) -> Int) -> Int:
    var acc = init
    for x in a:
        acc = op(acc, x)
    acc

fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var total = 0
    for r in 0..reps:
        total ^= fold_with(a, r, (acc, x) => (acc + x * 7) ^ (acc >> 3))
    total

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
