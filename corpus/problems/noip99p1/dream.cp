// NOIP 1999 Missile Interception: the most missiles one system stops is the longest
// non-increasing subsequence; the fewest systems for all of them is, by Dilworth, the longest
// strictly increasing one.
let heights = read_all<0..=50_000>()
print(heights.lis(strict: false, by: x => desc(x)).len)
print(heights.lis().len)
