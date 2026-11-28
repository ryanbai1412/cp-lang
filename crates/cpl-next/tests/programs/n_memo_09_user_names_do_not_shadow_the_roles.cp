// expect: ok, output "832040 1 2 3"
// rule: R06.4.1
// `@memo` is the decorator and `get`/`set` are `MemoTable`'s members by declaration, not by
// their text: a program's own `memo`, `get` and `set`, and another trait's `get`, change nothing.
fn memo(x: Int) -> Int = x
fn get(x: Int) -> Int = x + 1
fn set(x: Int) -> Int = x + 2
trait Store:
    fn get(self) -> Int
    fn set(mut self, v: Int)
@memo_table(slow)
struct Slow<K: Eq, V>:
    var ks: List<K> = []
    var vs: List<V> = []
impl<K: Eq, V> MemoTable<K, V> for Slow<K, V>:
    fn get(self, k: K) -> ?V:
        for i in 0..ks.len:
            if ks[i] == k: return vs[i]
        None
    fn set(mut self, k: K, v: V):
        ks.push(k)
        vs.push(v)
@memo(slow)
fn fib(n: Int) -> Int = if n < 2: n else: fib(n - 1) + fib(n - 2)
print(fib(30), memo(1), get(1), set(1))
