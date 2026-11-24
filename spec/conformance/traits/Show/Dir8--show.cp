// expect: ok, output "up-left down-left"
use std.grid.{Dir8}
print(Dir8.all()[7], Dir8.all()[1].back)
