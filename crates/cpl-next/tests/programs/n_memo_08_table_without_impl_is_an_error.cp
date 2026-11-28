// expect: E0327
// rule: R06.4.1
// A `@memo_table(word)` struct is the table only through its impl of `MemoTable`: `get` and `set`
// written on the struct itself fill nothing.
@memo_table(slow) // <- here
struct Slow<K: Eq, V>:
    var ks: List<K> = []
    var vs: List<V> = []
    fn get(self, k: K) -> ?V:
        for i in 0..ks.len:
            if ks[i] == k: return vs[i]
        None
    fn set(mut self, k: K, v: V):
        ks.push(k)
        vs.push(v)
@memo(slow)
fn sq(n: Int) -> Int = n * n
print(sq(2))
