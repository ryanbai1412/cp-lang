// `x in s` calls s's `contains`, generic or not: `Ords.contains<K>`
// infers K from the item.
struct Keys<K>(lo: K)
impl Keys<K>:
    pub fn contains(self, i: K) -> Bool = not (i < lo)
struct Ords(lo: Int)
impl Ords:
    pub fn contains<K>(self, i: K) -> Bool = i.ord >= lo
print(Int(3 in Keys(2)), Int(3 in Ords(2)))
