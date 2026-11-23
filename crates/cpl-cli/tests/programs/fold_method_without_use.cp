// R11.3.4: `a.fold(f, init)` is std.seq's method with no `use`, beside the
// listed free `std.algebra.fold`; `a.fold(+)` stays the built-in form.
let a = [3, 1, 2]
print(a.fold((acc, x) => acc + x, 0))
print(a.fold(+))
