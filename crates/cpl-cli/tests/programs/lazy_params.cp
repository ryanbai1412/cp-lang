// `lazy d: () -> T`: the caller writes the expression, the fn runs it with `d()` (decision 28.5).
var calls = 0
fn costly() -> Int:
    calls += 1
    7

fn or_else(a: ?Int, lazy d: () -> Int) -> Int:
    if let v = a:
        return v
    d()

fn either<T>(a: ?T, lazy d: () -> T) -> T:
    if let v = a:
        return v
    d()

fn check(ok: Bool, lazy why: () -> Str):
    if not ok:
        print("failed:", why())

let x: ?Int = 3
print(or_else(x, costly()), calls)
print(or_else(None, costly()), calls)
print(either(x, costly()), either(None, d: costly()), calls)
check(true, "never {costly()}")
check(false, "n = {calls}")
print(calls, or_else(None, () => 9))

struct Cache(v: ?Int)
impl Cache:
    fn get_or(self, lazy d: () -> Int) -> Int:
        if let x = self.v:
            return x
        d()
print(Cache(4).get_or(costly()), calls, Cache(None).get_or(costly()), calls)
