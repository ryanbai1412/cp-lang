// expect: ok, output "YES\nYES YES\nYES YES\nYES YES\nYES YES YES\nYES YES"
use std.strings.{RollingHash, HashVal}
print(HashVal(1, 2).hash_token == 2147483650)
var h = RollingHash("ab")
h.push('a')
print(h.all == h.hash_of("aba"), h.len == 3)
var g = RollingHash("abc")
g.pop()
print(g.all == g.hash_of("ab"), RollingHash("").is_empty)
let x = RollingHash("xab")
let c = RollingHash("abcd")
print(x.hash_of("ab") == x.get(1..3), c.concat([(hash: c.get(0..2), len: 2), (hash: c.get(2..4), len: 2)]) == c.all)
print(RollingHash("abcabd").lcp(0, 3) == 2, RollingHash("xabyab").lcs(3, 6) == 2, RollingHash("aaaa").lcp(0, 1) == 3)
print(RollingHash("abcd", bases: Some((131, 137))).lcp_with(1, RollingHash("xbcx", bases: Some((131, 137))), 1) == 2, x.hash_of("") == HashVal(0, 0))
