// expect: ok, output "1001 1 2"
// An element passed by value to a call that changes its list through `mut self` is the value
// it had at the call, whatever its index computes: the list may grow and move under it.
struct T<K>:
    var keys: List<K>
    fn grow(mut self, x: K) -> Int:
        for _ in 0..1000: keys.push(x)
        keys.len
    fn dup(mut self, i: Int) -> Int = self.grow(keys[i * 2])
var t = T<(Int, Int)>([(1, 2)])
print(t.dup(0), t.keys[1000])
