let n: 1..=100 = read()
let a: [0..n]Int = read()
fn total(k: Int) -> Int:
    var n = 0  // expect: shadows_top_level, loop_reduction
    for i in 0..k:
        n += a[i]
    return n
print(total(n))
