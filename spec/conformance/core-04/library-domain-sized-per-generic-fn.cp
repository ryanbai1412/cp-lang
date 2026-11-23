// expect: ok, output "807 3231"
// input: "3\n"
// rule: 04.2.26
// R04.2.26: a library domain over a fn's domain parameter is sized for each instance of the fn.
struct Sub<B>(bits: Int)
impl Index for Sub<B>:
    let count = 1 << B.count
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub<B> = Sub(k)
fn total<D: Domain>() -> Int:
    var dp: [Sub<D>]Int = 0
    for k in 0..Sub<D>.count:
        dp[Sub<D>.nth(k)] = k
    return Sub<D>.count * 100 + dp[Sub<D>.nth(Sub<D>.count - 1)]
let n: Int = read()
domain U = 0..n
domain W = 0..n + 2
print(total<U>(), total<W>())
