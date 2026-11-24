// R11.3.4: `a.fold_from(init, f)` is std.seq's method with no `use`, beside the
// listed free `std.algebra.fold`; `a.fold(+)` stays the built-in form.
let a = [3, 1, 2]
print(a.fold_from(0, (acc, x) => acc + x))
print(a.fold(+))
