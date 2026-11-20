// cp: 0.1 1f59ac0+dirty
// CCC 2003 S4 Substrings: count the distinct substrings of each string,
// the empty one included: hash every substring, one length at a time.
let t: 1..=10 = read()
for _ in 0..t:
    let s: [domain S]Char = read(alphabet: "ab01AB")
    let hash = StrHash(s)
    var total = 1                       // the empty substring
    for len in 1..=S.len:
        var hashes: List<Int> = []
        for l in S:
            if let r = l + (len - 1): hashes.push(hash.of(l, r))
        hashes.sort()
        total += 1 + count(hashes[k] != hashes[k - 1] for k in 1..hashes.len)
    print(total)
