// expect: E0203 at 3
// rule: 04.2.16
// A position has no `.ord`; its integer form is `Int(d)`.
enum D4 = Up | Right | Down | Left
for d in D4: print(d.ord)
