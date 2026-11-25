// expect: R0301
// An optional is a flag and a value (R02.4): `T` converts to `?T`, `??` picks the default
// only when there is no value, and `!` on `None` traps on its own line.
fn find(xs: List<Int>, k: Int) -> ?Int:
    for i in 0..xs.len:
        if xs.get(i) == k: return i
    None
var xs: List<Int> = List.new()
xs.push(5)
xs.push(7)
let a = find(xs, 7)
print(a ?? -1)
print(find(xs, 9) ?? -1)
print(a!)
var b: ?Int = None
print(b ?? 3)
b = 4
print(b!)
let c = find(xs, 1)
print(c!)
