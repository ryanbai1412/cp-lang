// expect: ok, output "1"
// An integer literal converts to `Mod`, so `= 0` fills a `[D]Z` with `Z(0)` (round 23, H10).
type Z = Mod<998_244_353>
var f: [0..=3]Z = 0
f[0] = Z(1)
print(f[0] + f[3])
