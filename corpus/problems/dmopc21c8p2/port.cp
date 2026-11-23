// Source: DMOPC '21 Contest 8 P2 (Kanna Market), https://dmoj.ca/problem/dmopc21c8p2
// Problem: a 2 x N grid, some cells known, the rest 0 (unknown); fill each unknown with 1..M so
//   that every 2 x 2 square has the same sum. Count the fillings mod 1e9 + 7. N, M <= 1e6.
// Solution: equal 2 x 2 sums means columns two apart have the same sum, so each parity class
//   picks one sum x and the classes are independent. A full column forces x, a half-known column
//   with value c allows c + 1..=c + M (a difference array counts how many columns allow x), and
//   each blank column has min(x - 1, 2M + 1 - x) fillings. A class gives the sum over allowed x of
//   ways(x)^blanks; multiply the two. O(N + M log N).
// Runnable port (proto/cp.py lib-run) of dmopc21c8p2.cp
// original: corpus/submissions/4522631.cpp

const MOD = 1_000_000_007
type Z = Mod<MOD>

let (n, m): (Int, Int) = read()
domain I = 0..n
domain S = 0..=(2 * m + 1)
let top: [I]Int = read()
let bot: [I]Int = read()

// Per parity class: how many columns are fully blank, how many constrain the
// column sum, and a difference array of the sums each constraint allows.
var blank: List<Int> = [0, 0]
var need: List<Int> = [0, 0]
var diff: List<[S]Int> = [[S] _ => 0, [S] _ => 0]

for i in I:
    let par = i.ord % 2
    let x = top[i] + bot[i]
    if top[i] == 0 and bot[i] == 0:
        blank[par] += 1
    else:
        let one_blank = top[i] == 0 or bot[i] == 0
        let lo = if one_blank: x + 1 else: x
        let hi = if one_blank: x + m else: x
        need[par] += 1
        diff[par][S(lo)] += 1
        diff[par][S(hi + 1)] -= 1

fn ways(x: Int) -> Int = max(0, min(x - 2, 2 * m - x) + 1)

var ans: Z = 1
for par in 0..2:
    var total: Z = 0
    var cover = 0
    for x in 0..=(2 * m):
        cover += diff[par][S(x)]
        if cover == need[par]:
            total += Z(ways(x)) ** blank[par]
    ans *= total
print(ans.v)
