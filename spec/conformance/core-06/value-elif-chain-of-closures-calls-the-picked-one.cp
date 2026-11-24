// expect: ok, output "9 -4 40 5 7"
// rule: 06.2.5
// An `elif` chain or a `match` with a binding arm, joining closures and fn
// locals, calls the one its condition picks, and only that one.
var calls = 0
fn tag(n: Int) -> Int:
    calls += 1
    n
fn choose(n: Int, k: Int) -> Int:
    fn twice(t: Int) -> Int = 2 * t + k - k
    let f = if n > 10: (t: Int) => t + k elif n < 0: (t: Int) => -t else: twice
    f(tag(n) - n + 4)
fn by(o: ?Int) -> Int:
    let w = 7
    let g = match o:
        Some(_) => (t: Int) => t * w
        None => (t: Int) => t + w
    g(1)
print(choose(20, 5), choose(-3, 1), choose(5, 2) * 5, calls + 2, by(Some(3)) * 0 + by(None) - 1)
