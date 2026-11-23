// expect: ok, output "0\n1"
// The subsets of an empty domain are one, the empty set.
domain E = 0..0
var n = 0
for s in Subset<E>:
    n += 1
    print(s.len)
print(n)
