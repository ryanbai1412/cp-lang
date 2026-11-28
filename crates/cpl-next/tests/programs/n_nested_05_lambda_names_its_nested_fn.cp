// expect: ok, output "1346269 89"
// rule: R05.10.5
// A lambda inside a nested fn that names the fn holds what the fn's group reads, so it rebuilds
// the fn from them; with `@memo` the name is the wrapper.
fn paths(n: Int, steps: List<Int>) -> Int:
    @memo(map)
    fn ways(i: Int) -> Int:
        if i == 0: return 1
        let via = (k: Int) => ways(k)
        var t = 0
        for s in steps:
            if s <= i: t += via(i - s)
        t
    ways(n)
fn plain(n: Int, steps: List<Int>) -> Int:
    fn ways(i: Int) -> Int:
        if i == 0: return 1
        let via = (k: Int) => ways(k)
        var t = 0
        for s in steps:
            if s <= i: t += via(i - s)
        t
    ways(n)
print(paths(30, [1, 2]), plain(10, [1, 2]))
