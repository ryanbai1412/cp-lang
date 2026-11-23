// expect: ok, output "1000000007"
// rule: 01.3.2
// A `const` is named like a variant; only `let` and `var` bindings must be lowercase.
const P = 1e9 + 7
print(P)
