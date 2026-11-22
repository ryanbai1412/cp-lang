// cp: 0.1 3bb81df+dirty
let n: 1..=2e5 = read()
domain I = 1..=n
let a: [I]0..=1e9 = read()
var best = 0
for i in I:
    for j in I:
        if i < j:
            best = max(best, a[i] + a[j])
print(best, sum(a[i] * a[j] % 7 for i in I for j in I))
