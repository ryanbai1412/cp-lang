// UCup 4 Stage 25 (Zagreb) H, tuned: port.cp's order, written as orig.cpp
// writes it. The two lists are one table indexed by the side, so each
// position is pushed without a branch; the median is read from the lists in
// place instead of building `left.rev + right`; the answer is printed in two
// pieces instead of concatenating slices; and the statement's length bound is
// written so positions are stored in 32 bits.
const N = 500_000

@!multitest
let s: Str = read()
let n: 1..=N = s.len
var by_side: [0..2]List<0..N> = []
for side in 0..2: by_side[side].reserve(n)
var side = 0
for i in (0..n).rev:
    by_side[side].push(i)
    if i > 0 and s[i] != s[i - 1]: side = if s[i - 1] > s[i]: 1 else: 0
let m = n / 2
let l = by_side[0].len
let del = if m < l: by_side[0][l - 1 - m] else: by_side[1][m - l]
print(s[0..del], s[del + 1..n], sep: "")
