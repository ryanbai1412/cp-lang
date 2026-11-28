// expect: E0503
// rule: 01.9.1
// has: "Wire"
// `@!multitest` is the one file directive; interactive mode is a `Wire` (R10.4, decision 667).
@!interactive // <- here
let wire = Wire()
let x: Int = wire.ask("? 1")
print("!", x)
