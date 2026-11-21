// CCO 2005 P4 Primed Sequences (CP Lang 0.2.2): shortest run of length >= 2
// with prime sum, breaking ties leftmost.
// original: corpus/submissions/4665767.cpp
use std.math.{is_prime}

@multitest
let n: Int = read()
let a: [0..n]Int = read()
var pre: [0..=n]Int = 0
for i in 0..n: pre[i + 1] = pre[i] + a[i]

for len in 2..=n:
    for l in 0..=n - len:
        if is_prime(pre[l + len] - pre[l]):
            print("Shortest primed subsequence is length {len}:", [a[i] for i in l..l + len])
            return
print("This sequence is anti-primed.")
