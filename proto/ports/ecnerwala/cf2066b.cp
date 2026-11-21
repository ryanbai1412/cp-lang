// Codeforces 2066 B: White Magic.
@!multitest
let n: 1..=200_000 = read()
domain I = 0..n
let a: [I]0..=1_000_000_000 = read()
let zeros = [i for i in I if a[i] == 0]
if let first = zeros.first:
    let b = [a[i] for i in I if a[i] != 0 or i == first]
    var seen: [0..=b.len]Bool = false
    var suffix_mex: [0..=b.len]Int = 0
    for i in (0..b.len).rev:
        if b[i] <= b.len: seen[b[i]] = true
        suffix_mex[i] = suffix_mex[i + 1]
        while seen[suffix_mex[i]]: suffix_mex[i] += 1
    let prefix_min = scan(b, min)
    let magical = all(prefix_min[i - 1] >= suffix_mex[i] for i in 1..b.len)
    print(n - zeros.len + Int(magical))
else:
    print(n)
