// expect: ok, output "0"
// `{}` where a Subset<D> is expected is the empty subset (search-dp mask DP).
domain B = 0..3
var s: Subset<B> = {}
print(s.len)
