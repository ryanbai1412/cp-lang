// expect: ok, output "3 3"
// rule: 05.10.8
// A local fn that calls itself through a lambda it passes on finds itself, memo or not.
fn app(f: Int -> Int) -> Int = f(1)
fn plain(n: Int) -> Int:
    fn ways(i: Int) -> Int:
        if i <= 0: return 1
        app(s => ways(i - s)) + app(s => ways(i - s - 1))
    ways(n)
fn memo(n: Int) -> Int:
    @memo(map)
    fn ways(i: Int) -> Int:
        if i <= 0: return 1
        app(s => ways(i - s)) + app(s => ways(i - s - 1))
    ways(n)
print(plain(2), memo(2))
