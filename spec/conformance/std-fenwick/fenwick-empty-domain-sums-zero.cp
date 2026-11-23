// expect: ok, output "0 0"
// rule: std.fenwick.Fenwick.new
domain E = 0..0
let f = Fenwick<E, Int>()
print(f.all, f.fold(E.all))
