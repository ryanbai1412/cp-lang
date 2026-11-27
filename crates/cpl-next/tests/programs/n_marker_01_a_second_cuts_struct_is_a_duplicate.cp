// expect: E0202
// rule: R06.4.12
// A marker names one declaration, whichever module writes it: core already marks its `Cut`,
// so the program's own `@cuts` struct is a second one.
@cuts // <- here
struct MyCut:
    k: Int
print(1)
