// A small function the interpreter inlines, whose body it hands to the
// tree-walker (`**`), reads its own parameters, not the caller's locals.
fn cube(a: Int) -> Int = a ** 3
fn two_to(a: Int) -> Int = 2 ** a
fn poly(a: Int, b: Int) -> Int = a ** 2 + b ** 2 + 1

print(cube(2))
print(two_to(10))
let x = 5
print(cube(x), poly(x, 3))
for i in 0..4:
    print(cube(i) + two_to(i))
