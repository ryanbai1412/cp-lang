// expect: ok, output "1\n0\n0"
// rule: R02.7.2, R02.7.4
enum Dir = N | E | S | W
fn north(d: Dir) -> Int = match d:
    N => 1
    _ => 0
print(north(Dir.N))
print(north(Dir.E))
print(north(Dir.W))
