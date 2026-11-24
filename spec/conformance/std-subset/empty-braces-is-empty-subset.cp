// expect: ok, output "0"
// gap: compiler 2026-11-24T04Z
// `{}` where a Subset<D> is expected is the empty subset (search-dp mask DP).
domain B = 0..3
var s: Subset<B> = {}
print(s.len)
