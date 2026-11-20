// CCO '18 P2 (Wrong Answer): construct a 100x100 upper-triangular cost matrix that
// breaks the greedy 2-pointer DP. Default-filling with 100 leaves only the 1-cost
// gadget edges to assign, and each row prints directly as an open slice.
// original: corpus/submissions/4801864.cpp

const N = 100
var a: [0..N, 0..N]Int = 100
a[0, 1] = 1
a[0, 2] = 1
a[0, 3] = 1
a[1, 2] = 1
for i in (2..N - 3).step(2):
    a[i, i + 3] = 1
    a[i + 1, i + 2] = 1

print(N)
for i in 0..N - 1:
    print(a.row(i)[i + 1..])
