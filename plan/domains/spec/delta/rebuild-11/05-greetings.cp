// Codeforces 1915F: Greetings
// I/O: t tests; each n then n lines a_i b_i (all 2n distinct); print the number of pairs where one segment strictly contains the other.
@!multitest
let n: 1..=2e5 = read()
let p = read<[n](a: -1e9..=1e9, b: -1e9..=1e9)>()
let (domain C, cb) = compress(p.map(q => q.b))
var fw = Fenwick<C, Int>()
var ans = 0
// by start: everyone already added starts earlier, and contains q when they end later
for q in p.sorted(by: q => q.a):
    let x = cb.at(q.b)
    ans += fw.fold(x.after)
    fw.add(x, 1)
print(ans)
