fn answers(s: Str, t: Str) -> List<Int>:
    let m = t.len
    let ws = [q + 1 for q in 0..m if t[q] == 'W']
    let gaps = [ws[c + 1] - ws[c] for c in 0..ws.len - 1].sorted()
    let gs = gaps.scan(+)
    var out: List<Int> = []
    var last_w = 0
    for i in 1..=s.len:
        if s[i - 1] == 'W':
            last_w = i
            continue
        let len = i - last_w
        if last_w > 0:
            out.push(m + 1 + (if ws.len > 0: max(0, ws.last! + len - 1 - m) else: 0))
        elif ws.len == 0:
            out.push(1)
        else:
            let c = gaps.partition_point(g => g >= len)
            out.push(1 + len + gs[c] + len * (gaps.len - c))
    out

@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
let s: Str = read()
let t: Str = read()
print(answers(s, t))
print(answers(t, s))
