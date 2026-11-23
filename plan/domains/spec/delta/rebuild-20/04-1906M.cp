// CF 1906M Triangle Construction. Each triangle needs three points that are
// not all on one side (they would be collinear), so it takes at most two from
// the fullest side: the answer is min(S / 3, S - max). S reaches 4e14.
let n: 3..=2e5 = read()
let a = read<[n]1..=2e9>()
let s = a.sum()
print(min(s / 3, s - a.max()!))
