// E. Memory Cell Interception
// https://codeforces.com/contest/2272/problem/E
// a read at s[i] (1-based) after its own last write p sees p + j for any j in 0..=m, and a write t[q] seen
// with i' in p..i-1 of s's commands before it, i.e. q + i'. Shifted by p this is [0, m] joined with the
// windows [q, q + i - p - 1], whose part above m comes from the last write in t alone. With no own
// write before it, it sees 0 or the union of windows of length i starting at t's writes.
@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
let s: Str = read()
let t: Str = read()

fn answers(s: Str, t: Str) -> List<Int>:
    let n = s.len
    let m = t.len
    let ws = [q + 1 for q in 0..m if t[q] == 'W']
    let gaps = [ws[j + 1] - ws[j] for j in 0..ws.len - 1].sorted()
    var res: List<Int> = []
    var p: ?Int = None
    var gi = 0
    var small = 0
    for i in 1..=n:
        if s[i - 1] == 'W':
            p = i
            continue
        if let pp = p:
            let len = i - pp
            res.push(m + 1 + (if let q = ws.last: max(0, q + len - 1 - m) else: 0))
        elif ws.len == 0:
            res.push(1)
        else:
            while gi < gaps.len and gaps[gi] <= i:
                small += gaps[gi]
                gi += 1
            res.push(1 + i + small + i * (gaps.len - gi))
    res

print(answers(s, t))
print(answers(t, s))
