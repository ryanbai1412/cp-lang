// Codeforces 2150 A — Incremental Path
//
// Person `i` starts at cell `1` and executes commands `s[0..=i]`, where `'A'`
// steps `+1` and `'B'` jumps to the next white cell, then paints their final
// cell black. Track `cur`, the cell where person `i` stands after replaying
// `s[0..i]`:
// - Step `s[i]` advances `cur += 1`, and if `s[i] == 'B'` skips black cells
//   to the next white cell, then paints `cur` black.
// - If `s[i] == 'B'`, person `i + 1` replaying `s[i]` will now skip the newly
//   painted `cur` (and any black cells after it), so we skip black cells again
//   to leave `cur` at the starting point for `s[i + 1]`.

@!multitest
let (n, m): (1..=100_000, 1..=100_000) = read()
let s: [0..n]Char = read()
let a: [0..m]1..=10 ** 9 = read()
var black = Set(a)

var cur = 1
fn skip_black():
    while cur in black: cur += 1

for ch in s:
    cur += 1
    if ch == 'B': skip_black()
    black.insert(cur)
    if ch == 'B': skip_black()

print(black.len)
print(black)
