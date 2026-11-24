// Σ over distinct substrings p of occ(p)². occ(p)² counts ordered pairs of suffixes that both
// start with p, so the answer is Σ LCP over ordered pairs of suffixes: n(n+1)/2 on the diagonal,
// plus twice the sum of minima over every window of the suffix array's adjacent-LCP array.
@!multitest
let s: Str = read()
let n = s.len

var stack: List<(h: Int, count: Int)> = []
var ending = 0   // Σ over starts of min(lcp[start..=here])
var pairs = 0
for h in suffix_array(s).lcp:
    var count = 1
    while let top = stack.pop_if(t => t.h >= h):
        count += top.count
        ending -= top.h * top.count
    stack.push((h, count))
    ending += h * count
    pairs += ending
print(n * (n + 1) / 2 + 2 * pairs)
