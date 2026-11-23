// expect: E0208 at 2
// rule: 01.3.2
// has: "const"
// An uppercase `let` reads as a type, so the error names the fix: `const NEG`, or a lowercase name (rebuild round 32 FL1).
let NEG = 1
print(NEG)
