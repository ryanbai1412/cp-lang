// expect: ok, output "120"
// rule: 04.2.27
// Walking a library domain visits every position once, in ordinal order: the ordinals of `Subset<0..4>` sum to 0 + 1 + … + 15.
domain V = 0..4
var t = 0
for s in Subset<V>:
    t += s.ord
print(t)
