// Codeforces 2207 E2 — N-MEX (Counting Version)
//
// `a[1..=n]` (with sentinel `a[0] = n`) must be non-increasing with
// `a[i] >= n - i` at every step, since removing `i` elements from `0..n` can
// lower the MEX by at most `i`.
//
// Scanning `i` from `n - 1` down to `0`, let `prv` be the current suffix MEX
// and `pool` the count of still-free values `< prv`:
// - If `cur = a[i] > prv`: the values in `(prv..cur)` are newly freed
//   (`pool += cur - prv - 1`), and position `i` can take any of those `pool`
//   values or any value in `cur..=n` (`n - cur + 1` choices), multiplying
//   `ans` by `pool + (n - cur + 1)`.
// - If `cur == prv`: position `i` must consume one of the `pool` free values
//   below `prv` (`ans *= pool`, `pool -= 1`).

type Z = Mod<1_000_000_007>

@!multitest
let n: 1..=200_000 = read()
var a: [0..=n]0..=10 ** 9 = n
for i in 1..=n: a[i] = read()

if not (1..=n).all(i => a[i] in (n - i)..=a[i - 1]):
    print(0)
    return

var ans: Z = 1
var prv = a[n]
var pool = a[n]
for i in (0..n).rev:
    let cur = a[i]
    if cur > prv:
        pool += cur - prv - 1
        prv = cur
        ans *= pool + (n - cur + 1)
    else:
        ans *= pool
        pool -= 1
print(ans)
