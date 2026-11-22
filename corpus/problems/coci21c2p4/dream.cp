// coci21c2p4 Magneti: place magnets on l slots so neighbours keep their distance. Insert magnets
// largest radius first into a set of components (connected-component DP): a magnet starts,
// extends or joins components, and `ends` counts whether the two line ends are already taken.
// `used` is the length forced so far; the leftover free slots are spread by stars and bars.
type Z = Mod<1_000_000_007>
let (n, l) = read<(1..=50, 1..=10_000)>()
let r = read<[n]1..=10_000>().sorted(by: desc)
let comb = Comb<Z>(l + n)
@memo(array)
fn ways(i: 0..=n, comps: 0..=n, used: 0..l, ends: 0..=2) -> Z:
    if i == n:
        return if comps == 1 and used == l - 1 and ends == 0: 1 else: 0
    fn next(dc: Int, du: Int, de: Int) -> Z:
        let (c, u, e) = (comps + dc, used + du, ends + de)
        if c in 0..=n and u in 0..l and e in 0..=2:
            return ways(i + 1, c, u, e)
        0
    let s = r[i]
    var total = (comps - 1 + ends) * next(1, 2 * s, 0) + ends * next(1, s, -1)
    if comps >= 1:
        total += (2 * comps - 2 + ends) * next(0, s, 0) + ends * next(0, 0, -1)
    if comps >= 2:
        total += (comps - 1) * next(-1, 0, 0)
    total
if n == 1:
    print(l)
else:
    print((0..l).map(u => ways(0, 0, u, 2) * comb.choose(u + n, n)).sum())
