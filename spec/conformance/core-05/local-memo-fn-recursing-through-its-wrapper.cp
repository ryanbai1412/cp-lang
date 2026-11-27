// expect: E0311
// rule: 06.4.1
// A memo fn spells its table (`@memo(array)` or `@memo(map)`); a bare `@memo` is an error.
fn paths(n: Int, steps: List<Int>) -> Int:
    @memo(map)
    fn ways(i: Int) -> Int:
        if i == 0: return 1
        steps.filter(s => s <= i).map(s => ways(i - s)).sum()
    ways(n)
fn fib(n: 0..=30) -> Int:
    @memo // <- here
    fn f(i: 0..=30) -> Int = if i < 2: i else: f(i - 1) + f(i - 2)
    f(n)
print("{fib(30)} {paths(1, [1, 2])} {paths(2, [1, 2])} {paths(3, [1, 2])}")
