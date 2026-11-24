let n: 1..=2000 = read()
let a: [0..n]Int = read()
var c = 0
for i in 0..n:
    for j in i..n:
        if a[i] + a[j] == 0:
            c += 1
for i in 0..n:
    for j in i + 1..n:
        for k in j + 1..n:
            if a[i] + a[j] + a[k] == 0:
                c += 1
var seen: List<Bool> = List.filled(n, false)
for i in 0..n:
    for v in 0..seen.len: seen[v] = false
    for j in i + 1..n:
        seen[j] = true
        if a[i] + a[j] == 0 and seen[0]:
            c += 1
print(c)
