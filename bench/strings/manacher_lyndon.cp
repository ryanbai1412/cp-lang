let n = 500_000
var s = List.filled(n, 0)
var x = 987654321
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = (x >> 16) & 3
let pal = manacher(s)
let lyn = lyndon_factorization(s)
let rot = min_rotation(s)
let (l, r) = pal.longest
print(pal.count + (r - l) + lyn.len + rot)
