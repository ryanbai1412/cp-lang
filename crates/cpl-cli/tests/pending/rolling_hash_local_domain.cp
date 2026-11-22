use std.strings.{HashVal}
// A RollingHash over a `[domain S]Char` read inside a loop: `get` takes ranges of `S`,
// closed, stepped and open-ended.
let t: 1..=10 = read()
for _ in 0..t:
    let s: [domain S]Char = read()
    let hash = RollingHash.new(s)
    var total = 1
    for len in 1..=S.len:
        var hashes: List<HashVal> = []
        for l in S:
            if let r = l + (len - 1): hashes.push(hash.get(l..=r))
        total += deduped(sorted(hashes)).len
    print(total, hash.get(S.first..) == hash.all)
