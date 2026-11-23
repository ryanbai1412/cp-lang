// 1760F Quests
// in: t; n c d; a_1..a_n (quest rewards). out: per test, the largest cooldown k, "Infinity" or "Impossible".
@!multitest
let (n, c, d) = read<(2..=2e5, 1..=1e16, 1..=2e5)>()
let top = read<[n]1..=1e9>().sorted(by: x => desc(x)).scan(+)

fn coins(k: Int) -> Int:
    let cycle = k + 1
    top.prefix(min(cycle, n)) * (d / cycle) + top.prefix(min(d % cycle, n))

if coins(d) >= c: print("Infinity")
elif coins(0) < c: print("Impossible")
else: print((0..=d).last_true(k => coins(k) >= c)!)
