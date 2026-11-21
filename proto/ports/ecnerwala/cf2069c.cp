@!multitest
type Z = Mod<998_244_353>

let n: 3..=200_000 = read()
var ones, open, ans: Z = 0
for a in read<[n]1..=3>():
    match a:
        1 => ones += 1
        2 => open = 2 * open + ones
        3 => ans += open
print(ans)
