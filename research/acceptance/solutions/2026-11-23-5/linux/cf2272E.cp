@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
let s: Str = read()
let t: Str = read()

fn answers(a: Str, b: Str) -> List<Int>:
    let ws = [q + 1 for q in 0..b.len if b[q] == 'W']
    var gaps = [ws[j + 1] - ws[j] for j in 0..ws.len - 1]
    gaps.sort()
    let pre = Prefix(gaps)
    var res: List<Int> = []
    var p = 0
    for i in 1..=a.len:
        if a[i - 1] == 'W':
            p = i
            continue
        if p > 0:
            let reach = if ws.len > 0: ws[ws.len - 1] + i - 1 - p else: 0
            res.push(max(b.len, reach) + 1)
        elif ws.len == 0:
            res.push(1)
        else:
            let cut = gaps.first_ge(i) ?? gaps.len
            res.push(1 + pre.prefix(cut) + i * (gaps.len - cut) + i)
    return res

print(answers(s, t))
print(answers(t, s))
