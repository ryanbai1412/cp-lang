// expect: E0606
// rule: 12.14.6
// `HashSet<K: Hash>`'s bound is met where the type is written: a struct whose own `Eq` has no own `Hash` can't key one.
struct E(x: Int)
impl Eq for E:
    fn eq(self, o: E) -> Bool = self.x % 2 == o.x % 2
let s: HashSet<E> = HashSet([E(1), E(3)])  // <- here
print(s.len)
