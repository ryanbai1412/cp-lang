// A getter on a struct with a `@cache` lends a field it gives, and one that
// fills the cache lends the cache after the fill; neither copies per read.
struct T:
    xs: List<Int>
    @cache c: List<Int>
    @get pub fn plain(self) -> List<Int> = xs
    @get pub fn cached(self) -> List<Int>:
        if c.is_empty: c = [x * 10 for x in xs]
        c
    @get pub fn first(self) -> List<Int>:
        if xs.is_empty: return [0]
        xs
let t = T([4, 5, 6], [])
var acc = 0
for i in 0..3: acc += t.plain[i] + t.cached[i]
let kept = t.cached
print(acc, kept, t.first.len, T([], []).first)
