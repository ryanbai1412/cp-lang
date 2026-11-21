@!multitest
let (n, k): (Int, Int) = read()
if k % 2 == 1:
    print(List.filled(k, n))
    return

var a: [0..k]Int = 0
var num_smaller = 0
for b in (0..=30).rev:
    if n.bit(b):
        if num_smaller < k: num_smaller += 1
        a[0..k] += 1 << b
        a[num_smaller - 1] -= 1 << b
    else:
        a[0..num_smaller / 2 * 2] += 1 << b
print(a)
