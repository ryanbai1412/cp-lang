// expect: ok, output "1000000007 2000000014 1000000007"
// A `const` is worked out at compile time by running it, and a domain whose ends read such
// constants hoists with them.
const P = 1e9 + 7
const Q = P * 2
domain D = 0..Q - P
print(P, Q, D.count)
