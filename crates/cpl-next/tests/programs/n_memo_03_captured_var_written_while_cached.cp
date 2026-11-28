// expect: E0311
// rule: R06.4.1
// A `@memo` fn keeps results computed from what it captures, so a captured `var` written while
// its cache lives, to the end of the block, is an error.
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
fn f(n: Int) -> Int:
    var k = 1
    @memo(slow)
    fn g(i: Int) -> Int = i * k
    k = 2 // <- here
    g(n)
print(f(3))
