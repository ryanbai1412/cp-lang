// expect: ok, output "832040 6 18"
// rule: R06.4.1 R05.10.8
// A `@memo` fn in a block keeps its table as a `var` of that block: each call of the fn around
// it starts with an empty cache, so the memo fn may read what it captures.
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
fn fib(n: Int) -> Int:
    @memo(slow)
    fn f(i: Int) -> Int = if i < 2: i else: f(i - 1) + f(i - 2)
    f(n)
fn scaled(n: Int, k: Int) -> Int:
    @memo(slow)
    fn g(i: Int) -> Int = if i == 0: 0 else: g(i - 1) + k
    g(n)
print(fib(30), scaled(3, 2), scaled(6, 3))
