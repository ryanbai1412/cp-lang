// Codeforces 2229 C1: sweep right-to-left flipping whenever the current sign
// is positive under the parity of suffix flips.
@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var ops: List<Int> = []
var flip = false
for i in (0..n).rev:
    if (a[i] > 0) != flip:
        ops.push(i + 1)
        flip = not flip
print(ops.len)
print(ops)
