// E. Memory Cell Interception
// https://codeforces.com/contest/2272/problem/E
// judge: generic

@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
let (s, t): (Str, Str) = read()

fn answers(mine: Str, other: Str) -> List<Int>:
    let len = other.len
    let w = [b + 1 for b in 0..len if other[b] == 'W']
    let gaps = [w[i + 1] - w[i] for i in 0..w.len - 1].sorted()
    var pre = [0]
    for g in gaps: pre.push(pre[pre.len - 1] + g)
    fn covered(l: Int) -> Int:
        if w.len == 0: return 0
        let small = gaps.first_ge(l) ?? gaps.len
        return l + pre[small] + (gaps.len - small) * l
    var out: List<Int> = []
    var a = 0
    for i in 1..=mine.len:
        if mine[i - 1] == 'W':
            a = i
        elif a == 0:
            out.push(1 + covered(i))
        else:
            let over = if w.len == 0: 0 else: max(0, w[w.len - 1] + (i - a) - 1 - len)
            out.push(len + 1 + over)
    return out

print(answers(s, t))
print(answers(t, s))

@alt(program)
fn brute():
    let (n, m): (1..=2e5, 1..=2e5) = read()
    let (s, t): (Str, Str) = read()
    var seen_s = [Set<Int>() for _ in 0..n]
    var seen_t = [Set<Int>() for _ in 0..m]
    fn go(i: Int, j: Int, cell: Int):
        let ts = i + j + 1
        if i < n:
            if s[i] == 'W': go(i + 1, j, ts)
            else:
                seen_s[i].add(cell)
                go(i + 1, j, cell)
        if j < m:
            if t[j] == 'W': go(i, j + 1, ts)
            else:
                seen_t[j].add(cell)
                go(i, j + 1, cell)
    go(0, 0, 0)
    print([seen_s[i].len for i in 0..n if s[i] == 'R'])
    print([seen_t[j].len for j in 0..m if t[j] == 'R'])

@gen
fn make_case(size: Int):
    let n = rand(1, min(size, 6))
    let m = rand(1, min(size, 6))
    print(1)
    print(n, m)
    print("".join(["RW"[rand(0, 1)] for _ in 0..n - 1]) + "R")
    print("R" + "".join(["RW"[rand(0, 1)] for _ in 0..m - 1]))
