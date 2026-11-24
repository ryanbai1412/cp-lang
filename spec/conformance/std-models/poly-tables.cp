// expect: ok, output "ok"
// input: "1 300\n"
// model: stirling1_row
// model: stirling2_row
// model: stirling1_col
// model: stirling2_col
// model: bell_table
// model: bernoulli_table
// model: partition_table
// std.poly's combinatorial tables against their recurrences, tabulated once: Stirling numbers
// by s(n, k) = s(n-1, k-1) + (n-1) s(n-1, k) and S(n, k) = S(n-1, k-1) + k S(n-1, k), Bell
// numbers as row sums of S, Bernoulli numbers by sum_j C(m+1, j) B_j = 0 and partitions by the
// coin-change loop over parts. Each op asks for a random size. Reads `seed n`; prints the first
// op that differs.
use std.poly.{stirling1_row, stirling2_row, stirling1_col, stirling2_col}
use std.poly.{bell_table, bernoulli_table, partition_table}
use std.mod.{Mod}
use std.rand.{Rng}

type Z = Mod<998_244_353>
const N = 90

var s1 = [List.filled(N + 2, Z(0)) for _ in 0..=N]
var s2 = [List.filled(N + 2, Z(0)) for _ in 0..=N]
s1[0][0] = Z(1)
s2[0][0] = Z(1)
for i in 1..=N:
    for j in 1..=i:
        s1[i][j] = s1[i - 1][j - 1] + Z(i - 1) * s1[i - 1][j]
        s2[i][j] = s2[i - 1][j - 1] + Z(j) * s2[i - 1][j]
var choose = [List.filled(N + 2, Z(0)) for _ in 0..=N + 1]
for i in 0..=N + 1:
    choose[i][0] = Z(1)
    for j in 1..=i: choose[i][j] = choose[i - 1][j - 1] + (if j < i: choose[i - 1][j] else: Z(0))
var bern = List<Z>()
for m in 0..=N:
    var s = Z(0)
    for j in 0..m: s += choose[m + 1][j] * bern[j]
    bern.push(if m == 0: Z(1) else: (Z(0) - s) / Z(m + 1))
var part = List.filled(N + 1, Z(0))
part[0] = Z(1)
for p in 1..=N:
    for v in p..=N: part[v] += part[v - p]

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let m = if rng.int(0, 3) == 0: rng.int(0, N) else: rng.int(0, 10)
    let c = rng.int(0, m + 3)
    if op == 0:
        let got: List<Z> = stirling1_row(m)
        if got != [s1[m][j] for j in 0..=m]: bad = "op {k}: stirling1_row({m})"
    elif op == 1:
        let got: List<Z> = stirling2_row(m)
        if got != [s2[m][j] for j in 0..=m]: bad = "op {k}: stirling2_row({m})"
    elif op == 2:
        let got: List<Z> = stirling1_col(c, m)
        if got != [if c <= i: s1[i][c] else: Z(0) for i in 0..=m]: bad = "op {k}: stirling1_col({c}, {m})"
    elif op == 3:
        let got: List<Z> = stirling2_col(c, m)
        if got != [if c <= i: s2[i][c] else: Z(0) for i in 0..=m]: bad = "op {k}: stirling2_col({c}, {m})"
    elif op == 4:
        let got: List<Z> = bell_table(m)
        if got != [sum(s2[i][j] for j in 0..=i) for i in 0..=m]: bad = "op {k}: bell_table({m})"
    elif op == 5:
        let got: List<Z> = bernoulli_table(m)
        if got != [bern[i] for i in 0..=m]: bad = "op {k}: bernoulli_table({m})"
    else:
        let got: List<Z> = partition_table(m)
        if got != [part[i] for i in 0..=m]: bad = "op {k}: partition_table({m})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
