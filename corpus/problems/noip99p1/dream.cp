// NOIP 1999 Missile Interception: the most missiles one system stops is the longest
// non-increasing subsequence; the fewest systems for all of them is, by Dilworth, the longest
// strictly increasing one.
let heights = read stream Int
print(heights.lis(by: desc, strict: false).len, heights.lis().len)
