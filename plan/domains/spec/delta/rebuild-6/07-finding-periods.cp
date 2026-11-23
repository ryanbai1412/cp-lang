// CSES 1733 Finding Periods: every period length of a string.
// Input: one string s. Output: all period lengths, increasing, on one line.
let s = read<Str>()
let z = z_function(s)
print([p for p in 1..=s.len if p == s.len or z[p] >= s.len - p])
