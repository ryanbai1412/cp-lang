// Runnable today/ port of CCC '23 S2 (Symmetric Mountains) for proto/cpc.
// original: corpus/submissions/5350805.cpp

fn main_solve():
    let n: Int = read()
    domain I = 0..n
    let h: [I]Int = read()
    domain L = 0..n
    var ans: [L]?Int = None
    for i in 0..n:
        for off in 0..=1:
            var cur = 0
            var j = 0
            while i - j >= 0 and i + j + off < n:
                cur += abs(h[I(i - j)] - h[I(i + j + off)])
                ans[L(2 * j + off)] min= cur
                j += 1
    var out: List<Int> = [ans[k]! for k in L]
    print(out)

main_solve()
