let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let top = a.max()!
print(a.filter(x => x < top).max() ?? -1)
