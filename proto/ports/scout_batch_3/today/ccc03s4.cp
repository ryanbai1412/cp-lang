// CCC 2003 S4 Substrings, as it has to be written today: every substring
// is hashed, one set per length. Builds; stress OK against 4182950.cpp.
@multitest
fn solve():
    let s: [domain S]Char = read()
    let hash = StrHash(s)
    var distinct = 1
    for len in 1..=S.len:
        var seen: Set<Int> = []
        for i in S:
            if let j = i + (len - 1): seen.insert(hash.of(span(i, j)))
        distinct += seen.len
    print(distinct)
