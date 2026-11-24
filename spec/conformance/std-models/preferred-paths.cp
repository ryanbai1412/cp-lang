// expect: ok, output "ok"
// input: "1 300\n"
// model: PreferredPaths
// PreferredPaths over seeded random rooted trees (rebuilt every 40 ops) against a plain array of
// preferred children: seeded random access, whose switches are checked bottom-up. Reads `seed n`.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..12
var rng = Rng(seed0)
var bad = ""
var k = 0
while k < n and bad == "":
    var perm = [V(i) for i in 0..12]
    rng.shuffle(mut perm)
    var par: [V]?V = None
    for i in 1..12: par[perm[i]] = Some(perm[rng.int(0, i - 1)])
    var pref: [V]?V = None
    let t = Rooted.from_parents(par)
    var pp = PreferredPaths(t)
    for _ in 0..40:
        if k >= n: break
        let v = V(rng.int(0, 11))
        var want: List<(lost: ?V, gained: ?V)> = []
        if let c = pref[v]:
            want.push((lost: Some(c), gained: None))
            pref[v] = None
        var x = v
        while true:
            var top = x
            while let p = par[top]:
                if pref[p] == Some(top): top = p else: break
            let Some(p) = par[top] else: break
            want.push((lost: pref[p], gained: Some(top)))
            pref[p] = Some(top)
            x = p
        if pp.access(v).collect() != want:
            bad = "op {k}: access {v}"
            break
        k += 1
print(if bad == "": "ok" else: bad)
