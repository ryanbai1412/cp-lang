// Codeforces 2229 C2: pick the single prefix pivot maximizing total gain from
// sign flips, then emit the right-to-left flip sequence up to that pivot.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var best_delta = 0
var best_idx: ?Int = none
var cur_delta = 0
for i in 0..n:
    if a[i] >= 0:
        cur_delta -= 2 * abs(a[i])
        if cur_delta > best_delta:
            best_delta = cur_delta
            best_idx = i
    cur_delta += 2 * abs(a[i])

var ops: List<Int> = []
if let idx = best_idx:
    var flip = false
    for i in (0..idx).rev:
        if (a[i] > 0) != flip:
            ops.push(i + 1)
            flip = not flip
    ops.push(idx + 1)
print(ops.len)
print(ops)
