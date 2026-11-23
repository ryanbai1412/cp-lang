let n: 1..=20 = read()
let a: [0..n]Int = read()
var best = 0
for mask in 0..(1 << n):  // expect: subset_masks_by_hand
    var s = 0  // expect: loop_reduction
    for i in 0..n:
        if (mask >> i) & 1 == 1:  // expect: manual_bit_test
            s += a[i]
    if s % 3 == 0:
        best = max(best, s)  // expect: chmin_chmax_if
print(best)
