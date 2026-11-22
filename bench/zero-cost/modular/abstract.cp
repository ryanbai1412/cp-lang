use std.mod.{Mod}

type Z = Mod<998244353>

fn kernel(n: Int, reps: Int, a: List<Int>) -> Int:
    var acc = Z.new(1)
    let c = Z.new(31)
    for r in 0..reps:
        let shift = Z.new(r + 1)
        for i in 0..n:
            acc = acc * c + Z.new(a[i]) + shift
    acc.v

let n: Int = read()
let reps: Int = read()
let a: [0..n]Int = read()
print(kernel(n, reps, a))
