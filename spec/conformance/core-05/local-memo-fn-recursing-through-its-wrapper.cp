// expect: ok, output "832040 1 2 3"
// rule: 05.10.8
fn paths(n: Int, steps: List<Int>) -> Int:
    @memo(map)
    fn ways(i: Int) -> Int:
        if i == 0: return 1
        steps.filter(s => s <= i).map(s => ways(i - s)).sum()
    ways(n)
fn fib(n: 0..=30) -> Int:
    @memo
    fn f(i: 0..=30) -> Int = if i < 2: i else: f(i - 1) + f(i - 2)
    f(n)
print("{fib(30)} {paths(1, [1, 2])} {paths(2, [1, 2])} {paths(3, [1, 2])}")
