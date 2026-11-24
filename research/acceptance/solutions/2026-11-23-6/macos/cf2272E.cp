@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
let s: Str = read()
let t: Str = read()

// reads of `own`: a write of `own` before the read gives m + 1 timestamps, a write of
// `other` at q gives [q + i0, q + p - 1] with i0 the last own write before read p
fn answers(own: Str, other: Str) -> List<Int>:
    let m = other.len
    let ws = [q + 1 for q in 0..m if other[q] == 'W']
    let gaps = [ws[i] - ws[i - 1] for i in 1..ws.len].sorted()
    let pre = gaps.scan(+)
    var out: List<Int> = []
    var i0 = 0
    for p in 1..=own.len:
        if own[p - 1] == 'W':
            i0 = p
            continue
        let len = p - i0
        if ws.len == 0:
            out.push(if i0 >= 1: m + 1 else: 1)
        elif i0 >= 1:
            out.push(m + 1 + max(0, ws.last! + len - 1 - m))
        else:
            let c = gaps.first_ge(len) ?? gaps.len
            out.push(1 + pre.cut(c) + len * (gaps.len - c) + len)
    out

print(answers(s, t))
print(answers(t, s))
