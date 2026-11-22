// Codeforces 2261 A — Binary String Span
//
// With `cnt == 1` `'1'`, no move is needed (`0`); with `cnt == 2`, one move
// suffices (`1`). With `cnt >= 3` `'1'`s, every position strictly between the
// leftmost `'1'` and rightmost `'1'` must be traversed, so the minimum cost
// equals the full span `last_one - first_one`.

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()
let cnt = s.count('1')
if cnt <= 2:
    print(cnt - 1)
else:
    let lo = (0..n).first(i => s[i] == '1')!
    let hi = (0..n).rev.first(i => s[i] == '1')!
    print(hi - lo)
