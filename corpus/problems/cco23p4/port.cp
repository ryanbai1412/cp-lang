// Source: CCO 2023 P4 (Flip it and Stick it), https://dmoj.ca/problem/cco23p4
// Problem: binary strings S (|S| <= 2e5) and T (|T| <= 3). One move reverses a substring of S;
//   print the fewest moves until S has no occurrence of T, or -1 if impossible.
// Solution: complement so T starts with 0, then a formula per pattern: "0" is 0 or -1; "00"
//   is #00, -1 if zeros > ones + 1; "01", "001", "011" need one move per occurrence; "010"
//   needs ceil(#010 / 2). "000" is -1 if zeros > 2 (ones + 1); otherwise greedy over the
//   zero runs between ones, moving zeros of runs > 2 into empty gaps two at a time, else
//   into gaps of size 1. O(n).
// original: corpus/submissions/5579929.cpp
let s: [domain S]Char = read()
let t: [domain T]Char = read()
let n = S.len
let m = T.len

var a: List<Int> = []
var ca: List<Int> = []
for i in S:
    let b = Int(s[i] == '1')
    a.push(b)
    ca.push(1 - b)

fn count1(w: List<Int>, b0: Int) -> Int:
    var res = 0
    for i in 0..n:
        if w[i] == b0: res += 1
    res

fn count2(w: List<Int>, b0: Int, b1: Int) -> Int:
    var res = 0
    for i in 0..n - 1:
        if w[i] == b0 and w[i + 1] == b1: res += 1
    res

fn count3(w: List<Int>, b0: Int, b1: Int, b2: Int) -> Int:
    var res = 0
    for i in 0..n - 2:
        if w[i] == b0 and w[i + 1] == b1 and w[i + 2] == b2: res += 1
    res

fn solve0(w: List<Int>) -> Int:
    if count1(w, 0) > 0: -1 else: 0

fn solve00(w: List<Int>) -> Int:
    let c0 = count1(w, 0)
    if c0 > n - c0 + 1: -1 else: count2(w, 0, 0)

fn solve000(w: List<Int>) -> Int:
    let c0 = count1(w, 0)
    if c0 > 2 * (n - c0 + 1): return -1
    var gaps: List<Int> = []
    var run = 0
    for i in 0..n:
        if w[i] == 1:
            gaps.push(run)
            run = 0
        else:
            run += 1
    gaps.push(run)

    var cnt0 = 0
    var cnt1 = 0
    for g in gaps:
        if g == 0: cnt0 += 1
        elif g == 1: cnt1 += 1

    var ans = 0
    for g0 in gaps:
        var g = g0
        while g > 2:
            if g == 3:
                if cnt1 > 0: cnt1 -= 1
                else:
                    cnt0 -= 1
                    cnt1 += 1
                g -= 1
            else:
                if cnt0 > 0:
                    cnt0 -= 1
                    g -= 2
                else:
                    cnt1 -= 1
                    g -= 1
            ans += 1
    ans

var ans = 0
if m == 1:
    ans = if t[T(0)] == '0': solve0(a) else: solve0(ca)
elif m == 2:
    let p0 = Int(t[T(0)] == '1')
    let p1 = Int(t[T(1)] == '1')
    if p0 == 0 and p1 == 0: ans = solve00(a)
    elif p0 == 1 and p1 == 1: ans = solve00(ca)
    elif p0 == 0 and p1 == 1: ans = count2(a, 0, 1)
    else: ans = count2(ca, 0, 1)
else:
    let p0 = Int(t[T(0)] == '1')
    let p1 = Int(t[T(1)] == '1')
    let p2 = Int(t[T(2)] == '1')
    if p0 == 0 and p1 == 0 and p2 == 0: ans = solve000(a)
    elif p0 == 1 and p1 == 1 and p2 == 1: ans = solve000(ca)
    elif p0 == 0 and p1 == 1 and p2 == 0: ans = (count3(a, 0, 1, 0) + 1) / 2
    elif p0 == 1 and p1 == 0 and p2 == 1: ans = (count3(ca, 0, 1, 0) + 1) / 2
    elif p0 == 0 and p1 == 0 and p2 == 1: ans = count3(a, 0, 0, 1)
    elif p0 == 1 and p1 == 1 and p2 == 0: ans = count3(ca, 0, 0, 1)
    elif p0 == 0 and p1 == 1 and p2 == 1: ans = count3(a, 0, 1, 1)
    else: ans = count3(ca, 0, 1, 1)
print(ans)
