// A program's own `get_or_insert` is the one its calls reach: this one
// stores over what the map held.
fn get_or_insert(mut self: Map<Int, Int>, k: Int, v: Int) -> Int:
    self[k] = v
    self[k]

var m = Map<Int, Int>()
print(m.get_or_insert(1, 5), m.get_or_insert(1, 9), m[1], m.len)
