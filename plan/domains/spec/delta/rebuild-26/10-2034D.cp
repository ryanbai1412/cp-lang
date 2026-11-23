// 2034D Darius' Wisdom: with count(0) <= count(2) (else mirror), fix the zero
// zone (1 move per 1, 2 per 2), then swap misplaced 2s and 1s: <= n moves.
@!multitest
let n: 1..=2e5 = read()
let raw = read<[n]0..=2>()
let flip = raw.count(x => x == 0) > raw.count(x => x == 2)
var a = [if flip: 2 - raw[n - 1 - i] else: Int(raw[i]) for i in 0..n]
let zeros = a.count(x => x == 0)
let ones = a.count(x => x == 1)
var moves = List<(Int, Int)>()
var free0 = [i for i in zeros..n if a[i] == 0]
var free1 = [i for i in zeros..n if a[i] == 1]
for i in 0..zeros:
    if a[i] == 1:
        let j = free0.pop()!
        moves.push((i, j))
        a[i] = 0
        a[j] = 1
        free1.push(j)
for i in 0..zeros:
    if a[i] == 2:
        let p = free1.pop()!
        moves.push((i, p))
        a[p] = 2
        let j = free0.pop()!
        moves.push((i, j))
        a[i] = 0
        a[j] = 1
        free1.push(j)
let mid = [i for i in zeros..zeros + ones if a[i] == 2]
let high = [i for i in zeros + ones..n if a[i] == 1]
for (i, j) in zip(mid, high): moves.push((i, j))
let shown = [if flip: (n - u, n - v) else: (u + 1, v + 1) for (u, v) in moves]
print(shown.len)
print(shown, sep: "\n")
