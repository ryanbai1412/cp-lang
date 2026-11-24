// expect: ok, output "12 abb 2 1 6 4"
// One generic fn called at several types, recursively and with a closure argument, gives each call its own instance (c8702d35d folds them into one C++ template).
fn twice<T>(x: T, f: T -> T) -> T = f(f(x))
fn count_eq<T: Eq>(xs: List<T>, v: T) -> Int = xs.count(x => x == v)
fn deep<T>(x: T, n: Int) -> Int = if n == 0: 0 else: 1 + deep(x, n - 1)
print(twice(3, x => x * 2), twice("a", s => s + "b"), count_eq([1, 2, 1], 1), count_eq(["x", "y"], "y"), deep(1, 6), deep("q", 4))
