// A beautiful subsequence is 1 2…2 3 with at least one 2. Scan once, counting
// subsequences that end in a 1, and those that end in "1 2+".
@!multitest
type Z = Mod<998_244_353>

let n: 3..=2e5 = read()
var ones, open, ans: Z = 0
for a in read<[n]1..=3>():
    match a:
        1 => ones += 1                 // start a new "1"
        2 => open = 2 * open + ones    // extend any "1 2*" by this 2, or start "1 2"
        3 => ans += open               // close every "1 2+"
print(ans)
