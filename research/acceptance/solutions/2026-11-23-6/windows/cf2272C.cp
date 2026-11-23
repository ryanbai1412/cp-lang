@!multitest
let n: 2..=2e5 = read()
let x: 0..=n = read()
let a: [0..n]0..=1e9 = read()

fn feasible(k: Int) -> Bool:
    var used = 0
    var cover = -1
    for i in 0..n:
        let lg = i > 0 and a[i] - a[i - 1] <= k
        let rg = i + 1 < n and a[i + 1] - a[i] <= k
        if lg or rg: continue
        if cover >= 0 and a[i] - cover <= k: continue
        used += 1
        cover = if a[i] + k <= 1e9: a[i] + k else: a[i] - 1
    used <= x

let hi = [a[i + 1] - a[i] for i in 0..n - 1].max()!
print((1..=hi).first_true(k => feasible(k))!)
