// expect: ok, output "1 mine YES"
// rule: 08.3.5
// A struct that writes its own `laws` keeps it: it is the struct's `laws`, the one `laws_hold`
// reads, and its `law`s derive none.
struct M:
    x: Int
    law small(a: M) = a.x < 2
    @get fn laws(self) -> List<(name: Str, sizes: List<M> -> List<Int>, holds: (List<M>, List<Int>) -> Bool)> = [(name: "mine", sizes: s => [s.len], holds: (s, p) => true)]
print(M(0).laws.len, M(0).laws[0].name, yes(laws_hold(M(0), [M(5)])))
