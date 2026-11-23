// CF 1860C Game on Permutation
// in: t; n, permutation p_1..p_n | out: per test the number of starting indices where Alice wins
@!multitest
let n: 1..=3e5 = read()
let p = read<[n]1..=n>()

var low = n + 1
var low_win = n + 1
var wins = 0
for x in p:
    if low < x and x < low_win:
        wins += 1
        low_win = x
    low min= x
print(wins)
