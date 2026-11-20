// Runnable today/ port of CCC '23 S1 (Trianglane) for proto/cpc.
// original: corpus/submissions/5350804.cpp

fn main_solve():
    let n: Int = read()
    domain Row = 0..2
    domain Col = 0..n
    let wet: [Row, Col]Int = read()
    var ans = 0
    for r in 0..2:
        for c in 0..n:
            if wet[Row(r), Col(c)] == 0: continue
            ans += 3
            if c > 0 and wet[Row(r), Col(c - 1)] == 1: ans -= 1
            if c + 1 < n and wet[Row(r), Col(c + 1)] == 1: ans -= 1
            if c % 2 == 0 and wet[Row(1 - r), Col(c)] == 1: ans -= 1
    print(ans)

main_solve()
