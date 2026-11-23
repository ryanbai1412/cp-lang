@!multitest
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
let s: Str = read()
let t: Str = read()

fn answers(s: Str, t: Str) -> List<Int>:
    let n = s.len
    let m = t.len
    let wt = [q for q in 0..m if t[q] == 'W']
    var gaps: List<Int> = []
    for j in 1..wt.len:
        gaps.push(wt[j] - wt[j - 1])
    gaps.sort()
    let pre = gaps.scan(+)
    var res: List<Int> = []
    var p = -1
    for i in 0..n:
        if s[i] == 'W':
            p = i
            continue
        if wt.len == 0:
            res.push(if p >= 0: m + 1 else: 1)
        elif p >= 0:
            res.push(m + 1 + max(0, wt[wt.len - 1] + i - p - m))
        else:
            let l = i + 1
            let c = gaps.partition_point(g => g >= l)
            res.push(1 + l + pre[c] + l * (gaps.len - c))
    res

print(answers(s, t))
print(answers(t, s))

@alt(program)
fn slow():
    var vs: [0..n]List<Int> = [0..n] _ => []
    var vt: [0..m]List<Int> = [0..m] _ => []
    for mask in 0..(1 << (n + m)):
        if popcount(mask) != n: continue
        var (i, j, cell) = (0, 0, 0)
        for pos in 0..n + m:
            if (mask >> pos) & 1 == 1:
                if s[i] == 'W': cell = pos + 1
                else: vs[i].push(cell)
                i += 1
            else:
                if t[j] == 'W': cell = pos + 1
                else: vt[j].push(cell)
                j += 1
    print([vs[i].distinct().len for i in 0..n if s[i] == 'R'])
    print([vt[j].distinct().len for j in 0..m if t[j] == 'R'])

@gen
fn make_case(size: Int):
    print(1)
    let n = rand(1, min(size, 7))
    let m = rand(1, min(size, 7))
    print(n, m)
    for len in [n, m]:
        var cs = [if rand(0, 1) == 0: 'R' else: 'W' for _ in 0..len]
        cs[rand(0, len - 1)] = 'R'
        print("".join(cs))
