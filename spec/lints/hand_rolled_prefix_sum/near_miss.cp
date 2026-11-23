let a = [1, 2, 3, 4]
var p = List.filled(a.len + 1, 0)
for i in 0..a.len:
    p[i + 1] = max(p[i], a[i])
print(p[3])
// A running total walked whole: a `Prefix` has no `any`, so the loop stays.
var pre: List<Int> = []
var run = 0
for x in a:
    run += x
    pre.push(run)
print(yes(pre.any(t => t > 5)))
