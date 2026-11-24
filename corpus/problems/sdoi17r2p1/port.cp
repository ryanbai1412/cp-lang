// Source: SDOI 2017 Round 2, Dragons and Dungeons (not on DMOJ; Luogu P3779)
// limits: 4000 ms, 512 MB
// Problem: T <= 10 tests: Y <= 2e5 dice with faces 0..X-1 (2 <= X <= 20) uniform; for 10
//   queries 0 <= A <= B <= (X-1)Y print P(A <= total <= B), absolute error 0.013579. At most
//   2 tests have Y > 800.
// Solution: for Y <= 800, exact: one die at a time, the next distribution averages X neighbouring
//   totals, a window over the previous row, O(X Y^2). Beyond that the central limit theorem
//   (the intended solution): the total is near normal with mean Y(X-1)/2, variance Y(X^2-1)/12,
//   so P(total <= k) = Phi((k + 1/2 - mean) / sd), Phi from the erf Taylor series. O(1) a query.
// original: corpus/submissions/5481972.cpp
// Bounds: faces: 2..=20; dice: 1..=2e5

fn normal_below(z: F64) -> F64:  // P(N(0, 1) <= z)
    if z < -6.0: return 0.0
    if z > 6.0: return 1.0
    // erf(x) = 2 / sqrt(pi) * sum over k of (-1)^k x^(2k+1) / (k! (2k+1)); |x| <= 4.3 keeps terms below 1e8
    let x = z / sqrt(2.0)
    var term = x  // (-1)^k x^(2k+1) / k!
    var sum = x
    for k in 1..120:
        term *= -x * x / F64(k)
        sum += term / F64(2 * k + 1)
    0.5 + sum / sqrt(3.141592653589793)

@!multitest
let (faces, dice): (2..=20, 1..=2e5) = read()
let top = dice * (faces - 1)
var below: List<F64> = [0.0]  // below[k]: the total is less than k (exact case only)
if dice <= 800:
    domain Total = 0..=top
    var chance: [Total]F64 = 0.0
    chance[Total(0)] = 1.0
    for die in 1..=dice:
        var next: [Total]F64 = 0.0
        var window = 0.0  // chance[t - faces + 1] + ... + chance[t]
        for t in Total.first!..=Total(die * (faces - 1)):
            window += chance[t]
            if let out = t.offset(-faces): window -= chance[out]
            next[t] = window / faces
        chance = next
    for t in Total: below.push(below.last + chance[t])
let mean = F64(top) / 2.0
let sd = sqrt(F64(dice * (faces * faces - 1)) / 12.0)
let at_most = (k: Int) => if dice <= 800: below[k + 1] else: normal_below((F64(k) + 0.5 - mean) / sd)
for _ in 0..10:
    let (lo, hi): (0..=top, 0..=top) = read()
    assert lo <= hi
    print(at_most(hi) - at_most(lo - 1), precision: 6)
