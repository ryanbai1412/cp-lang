use std.strings.{SuffixAutomaton}

domain Alpha = 0..4
let n = 200_000
var s = List<Alpha>()
var x = 246813579
for _ in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s.push(Alpha((x >> 16) % 4))
let sam = SuffixAutomaton<Alpha>.of(s)
let occ = sam.occ_counts()
var ans = sam.distinct
for i in 0..occ.len:
    ans += occ[i]
print(ans)
