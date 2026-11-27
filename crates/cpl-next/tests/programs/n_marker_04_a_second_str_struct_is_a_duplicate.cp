// expect: E0202
// rule: R06.4.12
// What a string literal is names one struct: std already marks its `Str`, so the program's own
// `@str` struct is a second one.
@str // <- here
struct Text:
    k: Int
print(1)
