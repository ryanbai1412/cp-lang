// expect: E0311 at 5
// rule: 05.10.8 06.4.1
fn f(n: Int) -> Int:
    var k = 1
    @memo(map)
    fn g(i: Int) -> Int = i * k
    k = 2
    g(n)
print(f(3))
