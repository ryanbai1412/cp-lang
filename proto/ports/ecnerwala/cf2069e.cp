// Codeforces 2069 E: A, B, AB and BA.
@!multitest
let s: Str = read()
let (a, b, ab, ba): (Int, Int, Int, Int) = read()

let cnt_a = s.count('A')
let cnt_b = s.len - cnt_a
var need = max([0, cnt_a - a, cnt_b - b])
if need > ab + ba:
    print("NO")
    return

var wild = 0
var ab_runs = List<Int>()
var ba_runs = List<Int>()
var start = 0
for i in 0..s.len:
    if i + 1 == s.len or s[i + 1] == s[i]:
        let len = i - start + 1
        if len % 2 == 1:
            wild += len / 2
        elif s[start] == 'A':
            ab_runs.push(len / 2)
        else:
            ba_runs.push(len / 2)
        start = i + 1

need -= wild
let sum_ab = sum(ab_runs)
let sum_ba = sum(ba_runs)
if sum_ab + sum_ba < need:
    print("NO")
    return

if min(sum_ab, ab) + min(sum_ba, ba) >= need:
    print("YES")
    return

var (over, cap, rem, other) = if sum_ab > ab:
    (ab_runs, ab, sum_ab, min(sum_ba, ba))
else:
    (ba_runs, ba, sum_ba, min(sum_ab, ab))
over.sort()
for l in over.rev:
    rem -= l
    need -= l - 1
    if min(rem, cap) + other >= need:
        print("YES")
        return
print("NO")
