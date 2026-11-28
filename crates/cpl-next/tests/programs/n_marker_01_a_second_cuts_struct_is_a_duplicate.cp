// expect: E0202
// rule: R06.4.12
// A role takes one declaration, whichever module fills it: std already fills `cuts` with `Cut`,
// so the program's own `@role(cuts)` struct is a second one.
@role(cuts) // <- here
struct MyCut:
    k: Int
print(1)
