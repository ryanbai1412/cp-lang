// CEOI 2019 practice P1: count pairs of gaps, one between two of a's points and one between two of
// b's points, with equal length.
let n: 1..=1500 = read()
let m: 1..=1500 = read()
let (a, b): ([n]Int, [m]Int) = read()
let gaps = a.pairs().map((x, y) => y - x).tally()
print(b.pairs().map((x, y) => gaps[y - x]).sum())
