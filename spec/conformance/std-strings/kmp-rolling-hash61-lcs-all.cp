// expect: ok, output "4 2 1 0 2\nYES YES 2 2 2\n2"
// rule: 08.1.1
use std.strings.{KmpAutomaton, RollingHash61, lcs_all}
let k = KmpAutomaton<Char>("aba")
print(k.len, k.step(3, 'b'), k.link(3), k.find_all("ababa").collect())
let h = RollingHash61("abcabd", base: Some(U64(131)))
let h2 = RollingHash61("xbcx", base: Some(U64(131)))
print(
    yes(h.get(0..2) == h.get(3..5)),
    yes(h.combine(h.get(0..2), h.get(2..6), 4) == h.all),
    h.lcp(0, 3),
    RollingHash61("xabyab").lcs(3, 6),
    h.lcp(1, 1, of: h2),
)
print(lcs_all(["abcd", "xbcy", "bcz"]))
