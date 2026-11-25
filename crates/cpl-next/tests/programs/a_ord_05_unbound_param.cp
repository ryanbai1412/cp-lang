// expect: E0606
// rule: R08.1.2
fn less<T>(a: T, b: T) -> Bool = a < b // <- here
print(if less(1, 2): 1 else: 0)
