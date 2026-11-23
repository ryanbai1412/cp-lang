// expect: ok, output "8 6\n4 2"
// input: "3\n"
// rule: 04.2.26
// R04.2.26: a `count` that reads a type argument is fixed once that domain is sized, per instance, in a function body too.
struct Sub<B>(bits: Int)

impl Index for Sub<B>:
    let count = 1 << B.count
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub<B> = Sub(k)

fn half(n: Int) -> Int:
    domain W = 0..n - 1
    var dp: [Sub<W>]Int = 0
    for k in 0..Sub<W>.count:
        dp[Sub<W>.nth(k)] = k
    return Sub<W>.count * 10 + dp[Sub<W>.nth(2)]

let n: Int = read()
domain V = 0..n
var dp: [Sub<V>]Int = 0
for k in 0..Sub<V>.count:
    let s = Sub<V>.nth(k)
    dp[s] = s.ord * 2
print(Sub<V>.count, dp[Sub<V>.nth(3)])
let h = half(n)
print(h / 10, h % 10)
