// CF 1955G GCD on a grid: try each divisor of gcd(corner, corner); keep the largest one with
// a down/right path of multiples.
@!multitest
let n = read<1..=100>()
let m = read<1..=100>()
domain R = 1..=n
domain C = 1..=m
let a = read<[R, C]1..=1e6>()
let start = (R.first!, C.first!)
let goal = (R.last!, C.last!)

fn reach(d: Int) -> Bool:
    var ok: [R, C]Bool = false
    for r in R:
        for c in C:
            if a[r, c] % d != 0: continue
            var here = (r, c) == start
            if let u = (r, c).up: here = here or ok[u]
            if let w = (r, c).left: here = here or ok[w]
            ok[r, c] = here
    ok[goal]

print(divisors(gcd(a[start], a[goal])).filter(reach).max()!)
