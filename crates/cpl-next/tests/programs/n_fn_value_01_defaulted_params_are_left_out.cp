// expect: ok, output "11 12\nYES NO\n20"
// rule: R06.1.5
// A fn item whose trailing parameters have defaults fits a fn type that leaves them out: a call through the value fills them.
fn tag(x: Int, k: Int = 10) -> Int = x + k
fn twice(f: (Int) -> Int, x: Int) -> Int = f(f(x))
print([1, 2].map(tag))
print([true, false].map(yes))
print(twice(tag, 0))
