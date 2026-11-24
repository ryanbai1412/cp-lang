// expect: E0202
// rule: 02.11.4
// Two structs of one name and one arity are a duplicate, not an overload.
struct Pair<A, B>(a: A, b: B)
struct Pair<C, D>(c: C, d: D) // <- here
print(Pair(1, 2).a)
