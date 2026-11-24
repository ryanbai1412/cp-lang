// expect: ok, output "6"
// rule: 05.10.8
// std_functions: 1
fn f(n: Int) -> Int:
    let k = 2
    @memo(map)
    fn g(i: Int) -> Int = i * k
    g(n)
print(f(3))
