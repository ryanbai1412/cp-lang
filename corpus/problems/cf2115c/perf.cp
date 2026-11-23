// Codeforces 2115 C, tuned: port.cp's recurrences with two buffers swapped per
// round, as orig.cpp does, instead of a new grid per round, and `win` filled
// as a table instead of `@memo` recursion.
@!multitest
let n: 1..=20 = read()
let m: 1..=4_000 = read()
let p100: 0..=100 = read()
let p = F64(p100) / 100.0
let q = 1.0 - p
let hp: [0..n]1..=400 = read()

let shines = hp.min()! - 1
let singles = hp.sum() - n * hp.min()!
let slack = m - shines - singles
if slack < 0: return print(0.0)

var level: [0..=shines]F64 = 0.0
var lv: [0..=shines, 0..n]F64 = 0.0
var nx: [0..=shines, 0..n]F64 = 0.0
lv[0, 0] = 1.0
for r in 0..=slack + shines:
    if r >= slack: level[r - slack] = lv[r - slack, 0]
    unchecked:
        nx[0, 0] = p * lv[0, 0] + q
        for k in 1..n: nx[0, k] = p * lv[0, k] + q * lv[0, k - 1]
        for s in 1..=shines:
            nx[s, 0] = p * lv[s - 1, 0] + q * max(lv[s - 1, n - 1], lv[s, 0])
            for k in 1..n: nx[s, k] = p * lv[s - 1, k] + q * lv[s, k - 1]
    swap(mut lv, mut nx)

var flat: [0..=singles]F64 = 0.0
var fl: [0..=singles]F64 = 0.0
var fx: [0..=singles]F64 = 0.0
fl[0] = 1.0
for r in 0..=slack + singles:
    if r >= slack: flat[r - slack] = fl[r - slack]
    unchecked:
        fx[0] = fl[0]
        for k in 1..=singles: fx[k] = p * fl[k] + q * fl[k - 1]
    swap(mut fl, mut fx)

var win: [0..=singles, 0..=shines]F64 = 0.0
for s in 0..=shines: win[0, s] = level[s]
for k in 1..=singles:
    win[k, 0] = flat[k]
    unchecked:
        for s in 1..=shines: win[k, s] = p * win[k, s - 1] + q * win[k - 1, s]
print(win[singles, shines])
