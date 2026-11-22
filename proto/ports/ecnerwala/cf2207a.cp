// Codeforces 2207 A — 1-1
//
// Operations only flip the middle bit of `101 <-> 111`, so maximal runs of `1`s
// separated by at most one `0` merge into independent contiguous `1`-blocks
// with fixed endpoints.
// - First pass (`101 -> 111`): fills every sandwiched `0`, reaching the unique
//   maximum `1`-count.
// - Second pass (`111 -> 101` left-to-right): reduces each length-`L` block of
//   `1`s to `1010...1` with `L / 2 + 1` ones, which is minimum because no two
//   adjacent `0`s can be created inside a block.

@!multitest
let n: 3..=100 = read()
var s: [0..n]Char = read()
fn sandwiched(i: Int) -> Bool: s[i] == '1' and s[i + 2] == '1'

for i in 0..n - 2 if sandwiched(i): s[i + 1] = '1'
let max_cnt = s.count('1')
for i in 0..n - 2 if sandwiched(i): s[i + 1] = '0'
print(s.count('1'), max_cnt)
