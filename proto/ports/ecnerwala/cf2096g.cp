@!interactive
@!multitest
let n: Int = read()
var q = 1
var ways = 1
while ways < n:
    q += 1
    ways *= 3

var left: [0..q]List<Int> = []
var right: [0..q]List<Int> = []
var off: [0..q]List<Int> = []
for pair in 1..=n / 2:
    let (a, b) = (2 * pair - 1, 2 * pair)
    var v = pair
    var total = 0
    for k in 0..q:
        let r = (if k == q - 1: -total else: v) % 3
        let d = if r == 2: -1 else: r
        if d == 1:
            left[k].push(a)
            right[k].push(b)
        elif d == -1:
            left[k].push(b)
            right[k].push(a)
        else:
            off[k] += [a, b]
        total += d
        v = (v - d) / 3
if n % 2 == 1:
    for k in 0..q: off[k].push(n)

print(q)
for k in 0..q:
    print(left[k].len + right[k].len, left[k], right[k])

let s: Str = read()
var hits: [1..=n]Int = 0
for k in 0..q:
    let side: List<Int> = match s[k]:
        'L' => left[k]
        'R' => right[k]
        'N' => off[k]
        _ => 1..=n
    for c in side: hits[c] += 1
print((1..=n).first(c => hits[c] == q)!)
