// expect: ok, output "5\n1\n3"
// `None` is an ordinary prelude name (R01.3.6), so a parameter named `None` hides the empty
// optional: as a value, beside `==`, and as an `if` branch's value.
fn plus(None: Int) -> Int = None + 1
fn same(None: Int, x: Int) -> Bool = None == x
fn pick(None: Int) -> Int = if None > 5: 0 else: None
print(plus(4))
print(if same(2, 2): 1 else: 0)
print(pick(3))
