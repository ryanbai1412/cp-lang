// expect: ok, output "4 6"
// input: "3\n"
// rule: 04.2.26
// R04.2.26: a generic `count` that reads no type argument is sized once what it reads is set, not before `read()`.
let n: Int = read()
struct G<B>(i: Int)
impl Index for G<B>:
    let count = 12 / n
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> G<B> = G(k)
domain V = 0..3
var a: [G<V>]Int = 0
for k in 0..G<V>.count: a[G<V>.nth(k)] = k * 2
print(G<V>.count, a[G<V>.nth(3)])
