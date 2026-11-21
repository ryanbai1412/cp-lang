// A chain of n calls whose frames hold values live across the call:
// deeper than an 8 MB stack holds.
let n: Int = read()
fn depth(k: Int) -> Int:
    if k == 0: return 0
    let a = k * 3 % 1000003
    let b = k * 5 % 1000033
    let c = k * 7 % 1000037
    let d = k * 11 % 1000039
    let e = k * 13 % 1000081
    let below = depth(k - 1)
    return (below * 31 + a * b % 1000099 + c * d % 1000117 + e) % 1000000007
print(depth(n))
