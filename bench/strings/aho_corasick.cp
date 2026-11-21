use std.strings.{AhoCorasick}

domain Alpha = 0..4
var x = 112233445
var pats: List<List<Alpha>> = []
for _ in 0..20_000:
    var w = List.filled(10, Alpha(0))
    for j in 0..10:
        x = (x * 1103515245 + 12345) & 0x7fffffff
        w[j] = Alpha((x >> 16) % 4)
    pats.push(w)
let ac = AhoCorasick<Alpha>.new(pats)
var st = ac.root
var hits = 0
for _ in 0..500_000:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    st = ac.step(st, Alpha((x >> 16) % 4))
    if ac.hit(st): hits += 1
print(ac.len, hits)
