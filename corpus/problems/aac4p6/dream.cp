// Source: An Animal Contest 4 P6 (Cozy Cottages), https://dmoj.ca/problem/aac4p6
// Problem: N <= 1e6 cottages on a circle with a_i <= 1e9 elves. Each of K <= 1e9 hours every
//   non-empty cottage sends exactly one elf to the next cottage, all at once. Print the final
//   counts.
// Solution: after min(N, K) hours either no cottage is empty, and then nothing changes any
//   more, or the elves stand at most one per cottage and the pattern turns one cottage per
//   hour: simulate m = min(N, K) hours and, if K > N with an empty cottage, rotate by K mod N.
//   The m hours take one O(N + m) sweep: a deque holds the hours at which no elf enters the
//   current cottage (seeded by running the elves of cottages N-1, N-2, ... in reverse); each
//   such hole costs the cottage an elf, and its own a_i elves fill the next a_i holes of the
//   stream it passes on.
// original: corpus/submissions/4227368.cpp
let (n, k): (1..=1e6, 0..=1e9) = read()
domain I = 0..n
var a: [I]0..=1e9 = read()
let m = min(n, k)

if k > 0:
    // the hours at which no elf enters the current cottage, from the stream it receives
    var holes = Deque<Int>()
    var carry = 0
    for j in 0..m:
        carry += a[I(n - 1 - j)]
        if carry > 0: carry -= 1 else: holes.push(j)
    for i in I:
        let x = a[i]
        holes.pop_while(h => h >= m - i.label)
        let ends_here = holes.last.is_some_and(h => h == m - i.label - 1)
        a[i] = max(x - holes.len, if ends_here: 0 else: 1)
        holes.push_front(-i.label - 1)
        for _ in 0..min(x, holes.len): holes.pop_front()
    // with an empty cottage the elves stand one per cottage and the pattern only turns
    if k > n and a.min()! == 0: a.rotate_right(k % n)
print(a)
