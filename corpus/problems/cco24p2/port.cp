// Source: CCO 2024 Day 1 P2, Pizza Party, https://dmoj.ca/problem/cco24p2
// limits: 4000 ms, 1024 MB
// Problem: pizzas a_1..a_N leave a cart top first, each onto a new stack or the top of one;
//   then people b_1..b_N each take a pizza of their flavour from some stack top (N <= 1e6,
//   flavours <= N). Print the fewest stacks K, each pizza's stack and each person's stack, or
//   -1 if impossible.
// Solution: partial: subtasks 1-3 only (flavours <= 2, or distinct a: the judged ones, the
//   statement notes 4-5 may have no efficient solution); other input prints nothing. Flavour
//   counts must match. Flavours <= 2: one stack if b is a reversed, else stack = flavour.
//   Distinct a: a person must take a pizza laid later than
//   the next person's, so cover b by chains decreasing in placement order: greedy patience
//   piles, binary-searching the first pile top below the new placement. O(N log N).
// original: corpus/submissions/6435481.cpp
// Bounds: n: 1..=1_000_000; x: 1..=n
let n: 1..=1_000_000 = read()
domain I = 0..n

var arra: [I]Int = 0
var arrb: [I]Int = 0
var cnt_a: [I]Int = 0
var cnt_b: [I]Int = 0
var max_a = 0
var max_b = 0

for i in 0..n:
    let x: 1..=n = read()
    let v = x - 1
    arra[I(i)] = v
    cnt_a[I(v)] += 1
    max_a max= v

for i in 0..n:
    let x: 1..=n = read()
    let v = x - 1
    arrb[I(i)] = v
    cnt_b[I(v)] += 1
    max_b max= v

var same_tally = true
var perm_ok = true
for i in 0..n:
    if cnt_a[I(i)] != cnt_b[I(i)]: same_tally = false
    if cnt_a[I(i)] != 1: perm_ok = false

if not same_tally:
    print(-1)
elif max_a <= 1 and max_b <= 1:
    var rev_eq = true
    for i in 0..n:
        if arra[I(i)] != arrb[I(n - 1 - i)]: rev_eq = false
    if rev_eq:
        print(1)
        var ones: List<Int> = List.filled(n, 1)
        print(ones)
        print(ones)
    else:
        print(2)
        var out_a: List<Int> = []
        var out_b: List<Int> = []
        for i in 0..n:
            out_a.push(arra[I(i)] + 1)
            out_b.push(arrb[I(i)] + 1)
        print(out_a)
        print(out_b)
elif perm_ok:
    var pos_rev: [I]Int = 0
    for i in 0..n:
        pos_rev[I(arra[I(n - 1 - i)])] = i

    var tops: List<Int> = []
    var sid: [I]Int = 0
    for i in 0..n:
        let val = arrb[I(i)]
        let cx = pos_rev[I(val)]
        let lo = (0..tops.len).first_true(mid => tops[mid] < cx) ?? tops.len
        if lo == tops.len:
            tops.push(cx)
        else:
            tops[lo] = cx
        sid[I(val)] = lo + 1

    print(tops.len)
    var out_a: List<Int> = []
    var out_b: List<Int> = []
    for i in 0..n:
        out_a.push(sid[I(arra[I(i)])])
        out_b.push(sid[I(arrb[I(i)])])
    print(out_a)
    print(out_b)
