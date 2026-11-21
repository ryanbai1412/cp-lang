@! multitest
let n: Int = read()
var q = 1
var ways = 1
while ways < n:
    q += 1
    ways *= 3
var left_pan: [0..q]List<Int> = []
var right_pan: [0..q]List<Int> = []
var neutral: [0..q]List<Int> = []
for i in 0..n / 2:
    var v = i + 1
    var tot = 0
    for k in 0..q:
        let raw = if k == q - 1: -tot else: v
        let rem = raw % 3
        let d = if rem == 2: -1 else: rem
        if d == 1:
            left_pan[k].push(2 * i)
            right_pan[k].push(2 * i + 1)
        elif d == -1:
            right_pan[k].push(2 * i)
            left_pan[k].push(2 * i + 1)
        else:
            neutral[k].push(2 * i)
            neutral[k].push(2 * i + 1)
        tot += d
        if k < q - 1:
            v = (v - d) / 3
if (n & 1) == 1:
    for k in 0..q:
        neutral[k].push(n - 1)
print(q)
for k in 0..q:
    var line: List<Int> = [left_pan[k].len + right_pan[k].len]
    for a in left_pan[k]:
        line.push(a + 1)
    for b in right_pan[k]:
        line.push(b + 1)
    print(line)
flush()
let s: Str = read()
var match_cnt: [0..n]Int = 0
for k in 0..q:
    let c = s[k]
    if c == 'L':
        for i in left_pan[k]:
            match_cnt[i] += 1
    elif c == 'R':
        for i in right_pan[k]:
            match_cnt[i] += 1
    elif c == 'N':
        for i in neutral[k]:
            match_cnt[i] += 1
    else:
        for i in 0..n:
            match_cnt[i] += 1
var ans = 0
while match_cnt[ans] != q:
    ans += 1
print(ans + 1)
flush()
