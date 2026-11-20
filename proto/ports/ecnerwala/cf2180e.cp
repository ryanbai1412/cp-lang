@!multitest
let (l, r0): (Int, Int) = read()
let r = r0 + 1
let cnt = min(ctz(r), ctz(l))
let split = bit_width(l ^ r) - 1
let half = 1 << split
let r_sz = r & (half - 1)
let l_sz = l & (half - 1)
var ans = 1
if l_sz + r_sz == half: ans += 1
print((ans << cnt) - 1)
