let n: 1..=5 = read()
let xs: [0..n]1..=9 = read()
let top = xs.sum() - n * xs.min()!
let low = xs.max()! - 1

@memo
fn paths(k: 0..=top, s: 0..=low) -> Int:
    if k == 0 or s == 0: return 1
    paths(k - 1, s) + paths(k, s - 1)
print(top, low, paths(top, low))
