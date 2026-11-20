// AAC 4 P6 — original: corpus/submissions/4227368.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, k): (1..=1_000_000, 0..=1_000_000_000) = read()
var a: [0..n]Int = read()

if k == 0:
    print(" ".join("{x}" for x in a))
    return

let m = min(n, k)
var dq: Deque<Int> = []
var csum = 0
for j in 0..m:
    csum += a[n - 1 - j]
    if csum > 0:
        csum -= 1
    else:
        dq.push(j)

for i in 0..n:
    let pi = a[i]
    while let x = dq.last and x >= m - i:
        dq.pop()                                // FRICTION 39.1: discarding ?Int pop result
    a[i] = max(0, a[i] - dq.len)
    if dq.last != m - i - 1:                    // FRICTION 39.1: ?Int != Int lifting unspecified
        a[i] max= 1
    dq.push_front(-i - 1)
    for _ in 0..min(pi, dq.len):
        dq.pop_front()

if k > n and any(x == 0 for x in a):
    rotate_left(mut a, (-k) % n)                // FRICTION 39.1: no rotate_right; relies on Euclidean (-k) % n

print(" ".join("{x}" for x in a))
