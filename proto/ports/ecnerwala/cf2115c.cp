// Codeforces 2115 C: Gellyfish and Eternal Violet.
//
// Track (s, k): s shines still needed and k single hits still needed. At the
// start s = min(h) - 1, the layers every monster loses together, and
// k = sum(h) - n * min(h), the hits that level the taller monsters. A shine
// lowers s and a miss is spent on a single hit, lowering k. A shine with
// s = 0 needed changes nothing. The one real choice is a miss at k = 0 with
// s > 0: start a new layer by hitting one monster (s - 1, and the other n - 1
// now need a hit each), or wait for a shine.
//
// While s > 0 and k > 0 every round makes progress, so with
// slack = m - (s0 + k0) spare rounds the rounds left are slack + s + k and
// need no dimension: `win(k, s)` is a plain recurrence. It leaves that region
// at k = 0 (`level[s]`) or s = 0 (`flat[k]`), where rounds can be wasted. Each
// edge is a DP over r rounds left, read off at r = slack + s or slack + k.
@!multitest
let n: 1..=20 = read()
let m: 1..=4_000 = read()
let p100: 0..=100 = read()
let p = F64(p100) / 100.0
let hp: [0..n]1..=400 = read()

let shines = hp.min()! - 1
let singles = hp.sum() - n * hp.min()!
let slack = m - shines - singles
if slack < 0: return print(0.0)

// Both edges of the (k, s) region are diagonals of a DP over rounds left:
// the chance from entry i is read after slack + i rounds.
fn diagonal<T>(size: Int, var dp: T, step: T -> T, at: (T, Int) -> F64) -> List<F64>:
    var out: List<F64> = []
    for r in 0..=slack + size:
        if r >= slack: out.push(at(dp, r - slack))
        dp = step(dp)
    out

// lv[s, k]: chance from (s, k < n) with r rounds left. A miss at k = 0 either
// starts a new layer, (s - 1, n - 1), or waits for a shine, (s, 0).
fn round(lv: [0..=shines, 0..n]F64) -> [0..=shines, 0..n]F64 =
    [0..=shines, 0..n] (s, k) => p * lv[max(s - 1, 0), k] + (1.0 - p) * miss(lv, s, k)
fn miss(lv: [0..=shines, 0..n]F64, s: 0..=shines, k: 0..n) -> F64 =
    if k > 0: lv[s, k - 1]
    elif s > 0: max(lv[s - 1, n - 1], lv[s, 0])
    else: 1.0
var lv0: [0..=shines, 0..n]F64 = 0.0
lv0[0, 0] = 1.0
let level = diagonal(shines, lv0, round, (lv, s) => lv[s, 0])

// fl[k]: with no shine needed, the chance of k misses in r rounds.
var fl0: [0..=singles]F64 = 0.0
fl0[0] = 1.0
let flat = diagonal(
    singles,
    fl0,
    fl => [0..=singles] k => p * fl[k] + (1.0 - p) * fl[max(k - 1, 0)],
    (fl, k) => fl[k],
)

@memo
fn win(k: 0..=singles, s: 0..=shines) -> F64 =
    if k == 0: level[s]
    elif s == 0: flat[k]
    else: p * win(k, s - 1) + (1.0 - p) * win(k - 1, s)
print(win(singles, shines))
