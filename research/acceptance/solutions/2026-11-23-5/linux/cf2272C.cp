@!multitest
let (n, x): (2..=2e5, 0..=2e5) = read()
let a: [0..n]0..=1e9 = read()

fn ok(k: Int) -> Bool:
    var lonely = [n].filled(false)
    for i in 0..n:
        let left = i > 0 and a[i] - a[i - 1] <= k
        let right = i + 1 < n and a[i + 1] - a[i] <= k
        lonely[i] = not left and not right
    var need = 0
    var i = 0
    while i < n:
        if lonely[i]:
            need += 1
            if i + 1 < n and lonely[i + 1] and a[i + 1] - a[i] <= 2 * k: i += 2
            else: i += 1
        else: i += 1
    return need <= x

print((1..=1e9).first_true(k => ok(k))!)
