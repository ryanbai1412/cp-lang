// expect: E0401
// rule: 06.5.1
// input: "2\n"
// Shared state a case changes must be a `var`; a shared `let` stays fixed (round 25, H09).
let seen = 0
@!multitest
seen += 1 // <- here
print(seen)
