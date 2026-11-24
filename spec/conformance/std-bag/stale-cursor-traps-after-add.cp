// expect: R0401 (checked)
// A B+tree add or remove may move keys between leaves, so a cursor made before it is stale:
// checked builds trap on its use, naming the change, and never read an unspecified key.
use std.bag.{SortedBag}
var b = SortedBag.of([1, 2, 3])
let c = b.cursor_first()!
b.add(0)
print(b.key_of(c))  // <- here
