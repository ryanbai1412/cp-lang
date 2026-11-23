// CF 1976D Invertible Bracket Sequences: (l, r) works iff bal[l-1] == bal[r] = v and
// the balance never exceeds 2v in between. Sparse table for the range max, binary search
// for the furthest end, then count equal balances by rank.
@!multitest
let s = read<Str>()
let n = s.len
var bal: [0..=n]Int = 0
for i in 0..n: bal[i + 1] = bal[i] + (if s[i] == '(': 1 else: -1)
let top = SparseTable(max, bal)

var at = [0..=n] _ => List<Int>()
var rank: [0..=n]Int = 0
for c in 0..=n:
    rank[c] = at[bal[c]].len
    at[bal[c]].push(c)

var ans = 0
for c in 0..=n:
    let v = bal[c]
    let far = (c..=n).last_true(e => top.fold(c..=e) <= 2 * v)!
    let end = (rank[c] + 1..at[v].len).first_true(i => at[v][i] > far) ?? at[v].len
    ans += end - rank[c] - 1
print(ans)
