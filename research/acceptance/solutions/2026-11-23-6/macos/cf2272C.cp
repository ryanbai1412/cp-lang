@!multitest
let (n, x): (2..=2e5, 0..=n) = read()
let a: [0..n]0..=1e9 = read()

fn needed(k: Int) -> Int:
    let lonely = [(i == 0 or a[i] - a[i - 1] > k) and (i == n - 1 or a[i + 1] - a[i] > k) for i in 0..n]
    var extra = 0
    var i = 0
    while i < n:
        if lonely[i]:
            extra += 1
            if i + 1 < n and lonely[i + 1] and a[i + 1] - a[i] <= 2 * k:
                i += 1
        i += 1
    extra

print((1..=1e9).first_true(k => needed(k) <= x)!)
