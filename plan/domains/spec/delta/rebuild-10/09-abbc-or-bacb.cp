// CF 1873G ABBC or BACB
// in: t; a string s over A and B | out: per test the most coins: all A's minus the shortest A-run between B's
@!multitest
let s = read<Str>()
let shortest = s.split('B').map(run => run.len).min()!
print(s.count(c => c == 'A') - shortest)
