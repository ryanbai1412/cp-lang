// expect: ok, output "5 6"
// rule: 01.3.2
// A `const` may be lowercase, like std's `inf` and `add` (decision 369).
const big = 5
print(big, big + 1)
