// expect: E0322
// rule: 03.2.9
// Holding itself through another struct is the same endless size.
struct A:
    b: ?B  // <- here
struct B:
    a: A
print(1)
