// CF 1994C, revised: print the answer for every prefix a[1..=i], not only the total.
// [l, r] is bad exactly when r is a reset point of the chain started at l. A chain that
// resets at r restarts at r + 1, so starts[s] counts the chains passing through start s.
@!multitest
let (n, x): (1..=2e5, 1..=1e9) = read()
let a = read<[n]1..=1e9>()

var nxt: [n]?Int = None
var r = 0
var sum = 0
for l in 0..n:
    if r < l:
        r = l
        sum = 0
    while r < n and sum + a[r] <= x:
        sum += a[r]
        r += 1
    if r < n: nxt[l] = Some(r)
    if r > l: sum -= a[l]

var starts: [0..=n]Int = 1
var bad: [n]Int = 0
for l in 0..n:
    if let Some(k) = nxt[l]:
        bad[k] += starts[l]
        starts[k + 1] += starts[l]
var total = 0
var answers = List<Int>()
for i in 0..n:
    total += (i + 1) - bad[i]
    answers.push(total)
print(answers)
