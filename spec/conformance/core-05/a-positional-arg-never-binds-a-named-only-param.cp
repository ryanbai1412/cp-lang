// expect: E0303
// rule: 08.1.4
// A positional argument never binds a named-only parameter, so the call
// stays the built-in row and names the key.
let xs = [3, 1, 3]
print(xs.sorted(x => -x)) // <- here
