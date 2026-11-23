// Codeforces 1907D: Jumping Through Segments
// I/O: t tests; each n then n lines l_i r_i; print the least k so moves of length <= k from 0 land in segment i after move i.
@!multitest
let n: 1..=2e5 = read()
let seg = read<[n](l: 0..=1e9, r: 0..=1e9)>()
fn reaches(k: Int) -> Bool:
    var lo = 0
    var hi = 0
    for s in seg:
        lo = max(lo - k, s.l)
        hi = min(hi + k, s.r)
        if lo > hi: return false
    true
print((0..=1e9).first_true(reaches)!)
