// expect: ok, output "set\n4\n4\nset\n4"
// rule: R06.4.1
// A block's `@memo` fn has its `f.clear()` too, found through the fn it clears.
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
fn run():
    @memo(slow)
    fn sq(n: Int) -> Int = n * n
    print(sq(2))
    print(sq(2))
    sq.clear()
    print(sq(2))
run()
