let s = "abacaba"
let sa = SuffixArray(s)
let h = RollingHash(s)
print(sa.sa, h.get(0..3) == h.get(4..7), palindromes(s).len)
