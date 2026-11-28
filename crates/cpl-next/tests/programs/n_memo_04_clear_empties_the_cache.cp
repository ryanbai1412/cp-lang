// expect: ok, output "set\n9\n9\nset\n9"
// rule: R06.4.1
// `f.clear()` empties a `@memo` fn's table, so its next call runs the body again.
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
        print("set")
        ks.push(k)
        vs.push(v)
@memo(slow)
fn sq(n: Int) -> Int = n * n
print(sq(3))
print(sq(3))
sq.clear()
print(sq(3))
