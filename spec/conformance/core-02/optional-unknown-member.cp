// expect: E0203
// rule: 02.4.1
// has: "zzz"
// An optional's members are its own (`map`, `filter`) and std's `self: ?T` fns; any other name is unknown, as on a struct.
let x: ?Int = Some(3)
print(x.zzz) // <- here
