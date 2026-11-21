// Codeforces 2261 A: with one '1' the answer is 0, with two it is 1, and with
// three or more it is the span from the first '1' to the last.
@!multitest
let _n: 1..=2e5 = read()
let s: Str = read()
let cnt = s.count('1')
print(if cnt <= 2: cnt - 1 else: s.rfind('1')! - s.find('1')!)
