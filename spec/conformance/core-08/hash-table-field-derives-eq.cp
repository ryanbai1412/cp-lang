// expect: ok, output "YES YES NO"
// rule: 08.1.6
// A struct whose fields all have `Eq` derives `==` field by field, a hash table field too.
struct Bag(name: Str, counts: HashMap<Int, Int>)
let xs = [3, 1, 3]
var a: HashMap<Int, Int> = HashMap()
a[1] += 2
var b: HashMap<Int, Int> = HashMap()
b[1] += 2
print(yes(xs.tally() == xs.tally()), yes(Bag("x", a) == Bag("x", b)), yes(Bag("x", a) == Bag("y", b)))
