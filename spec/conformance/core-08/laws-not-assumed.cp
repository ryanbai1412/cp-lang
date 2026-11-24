// expect: ok, output "abc"
// rule: 08.3.2
print(["a", "b", "c"].fold(Monoid<Str>("", (a, b) => a + b)))
