// 2004D Colored Portals, revised: city i stands at coordinate p_i (increasing, <= 1e9)
// through the nearest city (left or right of x) of any third kind.
@!multitest
let (n, q): (1..=2e5, 1..=2e5) = read()
domain City = 1..=n
let pos = read<[City]1..=1e9>()
let kind = read<[City]Str>()
let queries = read<[q](x: City, y: City)>()
let at = City.group(by: c => kind[c])
fn shares(s: Str, t: Str) -> Bool = s.any(ch => t.any(d => d == ch))
for (x, y) in queries:
    if shares(kind[x], kind[y]):
        print(abs(pos[x] - pos[y]))
        continue
    var best: ?Int = None
    for (t, spots) in at:
        if t == kind[x] or t == kind[y]: continue
        let cut = spots.partition_point(z => z >= x)
        for j in max(0, cut - 1)..min(spots.len, cut + 1):
            let z = spots[j]
            best min= abs(pos[x] - pos[z]) + abs(pos[z] - pos[y])
    print(best ?? -1)
