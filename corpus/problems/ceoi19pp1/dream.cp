# CEOI 2019 practice P1: count pairs of gaps, one between two of a's points and one between two of
# b's points, with equal length.
let (a, b) = read ([n]Int, [m]Int)
let gaps = Counter(a.pairs().map((x, y) => y - x))
print(b.pairs().sum(of: (x, y) => gaps[y - x]))
