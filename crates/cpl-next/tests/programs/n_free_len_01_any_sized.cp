// expect: ok, output "3 3 5 2 0\n2"
// `len(s)` is an ordinary std fn over `Sized`, so it counts a list, a string, a range, a set
// and a comprehension, and a user type with its own `Sized` impl.
struct Bag(n: Int)
impl Sized for Bag:
    @get fn len(self) -> Int = n
let xs = [4, 5, 6]
let e: List<Int> = []
print(len(xs), len("abc"), len(0..5), len([x for x in xs if x > 4]), len(e))
print(len(Bag(2)))
