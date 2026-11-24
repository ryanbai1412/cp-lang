// Source: DMOPC '21 Contest 8 P2 (Kanna Market), https://dmoj.ca/problem/dmopc21c8p2
// limits: 2000 ms, 256 MB
// Problem: a 2 x N grid, some cells known, the rest 0 (unknown); fill each unknown with 1..M so
//   that every 2 x 2 square has the same sum. Count the fillings mod 1e9 + 7. N, M <= 1e6.
// Solution: equal 2 x 2 sums means columns two apart have the same sum, so each parity class
//   picks one sum x and the classes are independent. A full column forces x, a half-known column
//   with value c allows c + 1..=c + M (a difference array counts how many columns allow x), and
//   each blank column has min(x - 1, 2M + 1 - x) fillings. A class gives the sum over allowed x of
//   ways(x)^blanks; multiply the two. O(N + M log N).
// original: corpus/submissions/4522631.cpp
// Per parity class: how many columns are fully blank, how many constrain the
// column sum, and a difference array of the sums each constraint allows.

const MOD = 1_000_000_007
type Z = Mod<MOD>

let n: 1..=1e6 = read()
let m: 1..=1e6 = read()
domain Col = 0..n
domain Parity = 0..2
domain Sum = 2..=2 * m + 1  // a column sum, plus one past the largest for the difference array
let top: [Col]0..=m = read()
let bot: [Col]0..=m = read()

// per parity class: the blank columns, the constraining ones, and a difference array
// counting the constraints each sum meets
var blank: [Parity]Int = 0
var need: [Parity]Int = 0
var allowed: [Parity, Sum]Int = 0
for i in Col:
    let par = Parity(i.ord % 2)
    let known = top[i] + bot[i]
    if top[i] == 0 and bot[i] == 0:
        blank[par] += 1
    else:
        let (lo, hi) = if top[i] == 0 or bot[i] == 0: (known + 1, known + m) else: (known, known)
        need[par] += 1
        allowed[par, Sum(lo)] += 1
        allowed[par, Sum(hi + 1)] -= 1

fn ways(x: Int) -> Int = min(x - 1, 2 * m + 1 - x)  // fillings of a blank column summing to x

var ans: Z = 1
for par in Parity:
    var total: Z = 0
    var cover = 0
    for x in Sum:
        cover += allowed[par, x]
        if cover == need[par] and x.label <= 2 * m: total += Z(ways(x.label)) ** blank[par]
    ans *= total
print(ans)
