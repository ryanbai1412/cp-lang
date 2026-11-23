// CF1936B Pinball. The ball bounces between the nearest '<' to its right and
// the nearest '>' to its left, alternating; the total time is a closed form in
// the sums of the k nearest of each side, read from prefix sums.
@!multitest
let n: 1..=5e5 = read()
let s = read<Str>()
var gt = List<Int>()        // 1-based cells holding '>'
var lt = List<Int>()        // 1-based cells holding '<'
for (k, ch) in s.items:
    if ch == '>': gt.push(k + 1) else: lt.push(k + 1)
let pg = gt.scan(+)
let pl = lt.scan(+)
var a = 0                   // '>' strictly left of i
var c = 0                   // '<' at or left of i
var out = List<Int>()
for (k, ch) in s.items:
    let i = k + 1
    if ch == '<': c += 1
    let b = lt.len - c      // '<' strictly right of i
    let lsum = (j: Int) => pg[a] - pg[a - j]
    let rsum = (j: Int) => pl[c + j] - pl[c]
    let t = if ch == '>':
        if b <= a: 2 * rsum(b) - 2 * lsum(b) - i + n + 1
        else: 2 * rsum(a + 1) - 2 * lsum(a) - i
    else:
        if a <= b: 2 * rsum(a) - 2 * lsum(a) + i
        else: 2 * rsum(b) - 2 * lsum(b + 1) + i + n + 1
    out.push(t)
    if ch == '>': a += 1
print(out)
