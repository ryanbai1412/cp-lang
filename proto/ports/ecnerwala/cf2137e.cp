@!multitest
let (n, k): (Int, Int) = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
var cnt: [0..=n]Int = 0
for x in a: cnt[x] += 1
var base = 0
while cnt[base] == 1: base += 1
if base >= n - 1:
    print(n * (n - 1) / 2)
elif cnt[base] >= 2 and k == 1:
    var mex = base
    while cnt[mex] > 0: mex += 1
    print(sum((if x < mex and cnt[x] == 1: x else: mex) for x in a))
else:
    let v = if (cnt[base] == 0) == (k % 2 == 0): base + 1 else: base
    print(sum((if x < base: x else: v) for x in a))
