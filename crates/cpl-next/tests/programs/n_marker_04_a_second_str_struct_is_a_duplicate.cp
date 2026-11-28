// expect: E0202
// rule: R06.4.12
// What a string literal is names one struct: std already fills it with `Str`, so the program's own
// `@role(str)` struct is a second one.
@role(str) // <- here
struct Text:
    k: Int
print(1)
