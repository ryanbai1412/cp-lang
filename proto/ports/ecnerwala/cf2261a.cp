// Codeforces 2261 A: with one '1' the answer is 0, with two it is 1, and with
// three or more it is the span from the first '1' to the last.
@!multitest
let n: 1..=2e5 = read()
let s: Str = read()
let ones = [i for i in 0..n if s[i] == '1']
print(if ones.len <= 2: ones.len - 1 else: ones.last! - ones.first!)
