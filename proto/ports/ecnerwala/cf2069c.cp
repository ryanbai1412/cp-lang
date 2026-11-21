@!multitest
type Z = Mod<998_244_353>

let n: 3..=200_000 = read()
var ones, open, ans: Z = (0, 0, 0)
repeat n:
    match read::<1..=3>():
        1 => ones += 1
        2 => open = 2 * open + ones
        3 => ans += open
print(ans)
