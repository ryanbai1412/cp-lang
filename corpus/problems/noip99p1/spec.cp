// noip99p1 Missile Interception: one system hits a non-increasing run of heights. How many missiles
// can one system hit (longest non-increasing subsequence), and how many systems hit them all
// (Dilworth: the longest strictly increasing subsequence)?
let h = read_all<0..=50_000>()
print(h.lis(strict: false, by: x => desc(x)).len)
print(h.lis().len)
