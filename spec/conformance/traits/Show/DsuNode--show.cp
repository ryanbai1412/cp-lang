// expect: ok, output "parent 2, size 3"
use std.dsu.{DsuNode}
print(DsuNode<0..3>(_parent: 2, _size: 3))
