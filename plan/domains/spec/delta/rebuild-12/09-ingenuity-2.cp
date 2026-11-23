// 1974D Ingenuity-2
// in: t; per test n, then a string of N/S/E/W. out: an R/H assignment string, or NO, per test
@!multitest
let n: Int = read()
let s: Str = read()
var seen: Counter<Char, Int> = Counter(0)
var who: List<Char> = []
for c in s:
    let rover_first = c == 'N' or c == 'S'
    who.push(if (seen[c] % 2 == 0) == rover_first: 'R' else: 'H')
    seen.add(c, 1)
let rovers = who.count(c => c == 'R')
if (seen['N'] - seen['S']) % 2 != 0 or (seen['E'] - seen['W']) % 2 != 0 or rovers == 0 or rovers == n:
    print("NO")
else:
    print(Str(who))
