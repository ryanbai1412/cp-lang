// Codeforces 2147 D, tuned: orig.cpp's sort and run-length scan in place of
// port.cp's Map of frequencies; the scoring is port.cp's. The scan is written
// out because `a.runs()` on a read `[0..n]1..=10 ** 9` fails to check.
@!multitest
let n: 1..=200_000 = read()
var a: [0..n]1..=10 ** 9 = read()
a.sort()

var extras: List<Int> = []
var i = 0
while i < n:
    var j = i + 1
    while j < n and a[j] == a[i]: j += 1
    if a[i] % 2 == 1: extras.push(j - i)
    i = j
extras.sort(desc: true)
let base = sum(x / 2 for x in a)
var score: [0..2]Int = base
for (k, c) in extras.items: score[k % 2] += c
print(score[0], score[1])
