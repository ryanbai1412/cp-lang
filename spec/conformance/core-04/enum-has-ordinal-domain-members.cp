// expect: ok, output "R 3 X L YES"
// rule: 04.2.13
// A fieldless enum is a domain: `try_at`, `wrap` and `nth` count its variants by ordinal.
enum Shape = L | R | X
print(Shape.try_at(1)!, Shape.all.len, Shape.wrap(-1), Shape.nth(0), yes(Shape.try_at(3) == None))
