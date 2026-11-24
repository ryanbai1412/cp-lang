// expect: E0321
// rule: 07.2.17
// A `Str` key never wraps either: `s[-2]` is E0321, whose help is `s[s.len - 2]` (decision 508).
let s = "abc"
print(s[-2]) // <- here
