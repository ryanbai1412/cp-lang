// expect: E0503
// rule: 02.7.1
// has: "name: Type"
// A payload field is `name: Type`, as a struct field is: `One(v: Int)`.
enum Boxed = One(Int) | Empty // <- here
print(0)
