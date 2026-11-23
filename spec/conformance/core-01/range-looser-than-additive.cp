// expect: ok, output "4 3"
// rule: 01.5
// `..` and `..=` bind looser than `+` and `-`: `0..=n - k` is `0..=(n - k)`, `i + 1..n` is `(i + 1)..n`.
let n = 5
let k = 2
let i = 1
print((0..=n - k).len, (i + 1..n).len)
