@!multitest
let n: Int = read()
var a: List<Int> = [read::<Int>() for _ in 0..n]
a.sort()
var base = 0
var extras: List<Int> = []
var i = 0
while i < n:
    var j = i
    while j < n and a[j] == a[i]: j += 1
    let cnt = j - i
    base += (a[i] / 2) * cnt
    if a[i] % 2 == 1: extras.push(cnt)
    i = j
extras.sort(by: x => -x)
var a0 = base
var a1 = base
for z in 0..extras.len:
    if z % 2 == 0: a0 += extras[z] else: a1 += extras[z]
print(a0, a1)
