// expect: ok, output "3 1 2"
// rule: 04.2.24
// input: "bab\n"
// A word reads straight into `[domain S]Letter`: each character is checked into `Letter`.
domain Letter = 'a'..='z'
let s: [domain S]Letter = read()
print(s.len, count(c == Letter('a') for c in s), count(c == Letter('b') for c in s))
