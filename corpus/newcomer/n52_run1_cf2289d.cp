// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2289D.
@!multitest
let n: 2..=2e5 = read()
domain I = 0..n
let a: [I]-1e9..=1e9 = read()
var s: [I]Int = 0
var run = 0
for i in I:  // expect: hand_rolled_prefix_sum
    run += a[i]
    s[i] = run
if s.any(x => x < 0):
    print(-1)
else:
    let (domain C, comp) = compress(s)
    var bit = Fenwick<C, Int>()  // expect: missing manual-inversion-count
    var inv = 0
    for i in I:
        let id = comp.at(s[i])
        inv += i.to_idx - bit.prefix(id.r)
        bit.add(id, 1)
    print(inv)
