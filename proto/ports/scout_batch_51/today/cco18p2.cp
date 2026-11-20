// Runnable today/ port of CCO '18 P2 (Wrong Answer) for proto/cpc.
// original: corpus/submissions/4801864.cpp

const N = 100
domain V = 0..N
var a: [V, V]Int = 100
a[V(0), V(1)] = 1
a[V(0), V(2)] = 1
a[V(0), V(3)] = 1
a[V(1), V(2)] = 1
var i = 2
while i + 3 < N:
    a[V(i), V(i + 3)] = 1
    a[V(i + 1), V(i + 2)] = 1
    i += 2

print(N)
for r in 0..N - 1:
    var row: List<Int> = []
    for c in r + 1..N:
        row.push(a[V(r), V(c)])
    print(row)
