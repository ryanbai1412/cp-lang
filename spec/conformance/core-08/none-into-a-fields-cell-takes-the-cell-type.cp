// expect: ok, output "1 1 9"
// rule: 08.2.11
// `adds[t] = None` and `self.keys[v] = None` write a field's cell, so the cell's type, `?T`
// or `?K`, says what `None` is an optional of, as it does for a local list.
struct S<D, T: Ord>:
    var adds: List<?T>
    var keys: [D]?T
    fn clear(mut self, t: Int, v: D):
        adds[t] = None
        self.keys[v] = None
domain V = 0..2
var s = S<V, Int>([Some(1), Some(2)], [V] _ => Some(9))
s.clear(1, V(0))
print(s.adds.count(a => a != None), s.keys.count(k => k != None), s.keys[V(1)] ?? 0)
