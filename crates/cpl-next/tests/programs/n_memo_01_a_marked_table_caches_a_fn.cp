// expect: ok, output "23416728348467685 601080390 3"
// rule: R06.4.1
// `@memo(word)` keeps its fn's results in the struct some module marks `@memo_table(word)`:
// a fn of one parameter keys it by that parameter, of more by their tuple.
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
@memo(slow)
fn grid(r: Int, c: Int) -> Int:
    if r == 0 or c == 0: return 1
    grid(r - 1, c) + grid(r, c - 1)
print(fib(80), grid(16, 16), fib(4))
