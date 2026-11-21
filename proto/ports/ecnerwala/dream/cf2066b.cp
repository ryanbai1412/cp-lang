// With no zero the whole sequence is magical. Otherwise at most one zero can
// stay, and the leftmost is the best one to keep: that subsequence is magical
// exactly when every prefix minimum is at least the MEX of what follows it.
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]0..=1e9>()
let first = a.find(0) ?? return print(n)
let b = [x for (i, x) in a.items() if x != 0 or i == first]
let (low, mex) = (b.prefix_min(), b.suffix_mex())
print(n - a.count(0) + Int((1..b.len).all(i => low[i - 1] >= mex[i])))
