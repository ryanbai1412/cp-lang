// CF 1849C Binary String Copying
// in: t; n m, binary string s, m lines l r | out: per test the number of distinct strings after sorting s[l..=r] in each copy
@!multitest
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain I = 1..=n
let s = read<[I]Char>()

var first_one: [I]?I = None
var one: ?I = None
for i in I.rev:
    if s[i] == '1': one = Some(i)
    first_one[i] = one
var last_zero: [I]?I = None
var zero: ?I = None
for i in I:
    if s[i] == '0': zero = Some(i)
    last_zero[i] = zero

var results: SortedSet<(Int, Int)> = SortedSet()
for _ in 0..m:
    let q = read<Range<I>>()
    let changed = match (first_one[q.first!], last_zero[q.last!]):
        (Some(lo), Some(hi)) if lo < hi => (Int(lo), Int(hi))
        _ => (0, 0)
    results.add(changed)
print(results.len)
