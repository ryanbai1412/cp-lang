// expect: ok, output "0 3\n2 -1 2 -4 5"
// rule: 07.11.3
// `group(by:)` walks keys in ascending order, and `%` is Euclidean, so negative values land on keys 0..3 (adversarial round 47).
for (k, xs) in [3, -1, 2, -4, 5].group(by: x => x % 3):
    print(k, xs)
