// Mock CCC '21 S2 (nccc8s2), runnable under proto/cp.py lib-run today
// (proto/cpc cannot iterate unknown-length Str). See ../nccc8s2.cp.
// original: corpus/submissions/3381135.java

let s: Str = read()
var cnt = List.filled(26, 0)
for i in 0..s.len:
    cnt[s[i] - 'a'] += 1
var ans: Mod<1_000_000_007> = 1
for i in 0..26:
    ans = ans * Mod<1_000_000_007>(cnt[i] + 1)
print(ans)
