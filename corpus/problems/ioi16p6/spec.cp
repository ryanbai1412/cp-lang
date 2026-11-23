// ioi16p6 Aliens: cover points with at most k diagonal squares of least total area. A point needs
// the diagonal range min(r, c)..=max(r, c); drop ranges inside others, and a photo covers a run of
// the rest. Pay `fee` per photo to lift the count limit (`aliens`); a photo from range j's left
// cut to range i's right cut costs (Int(i.r) - Int(j.l))², a line in Int(j.l) evaluated at
// Int(i.r). `aliens` needs the fewest photos among equal costs, so each line carries its count and
// the LiChao key (val, count) breaks ties by it.
let (n, m, k) = read<(1..=1e5, 1..=1e6, 1..=1e5)>()
domain Diag = 0..m
let spans = read<[n](r: Diag, c: Diag)>().map(p => min(p.r, p.c)..=max(p.r, p.c)).outermost()

struct Photo(line: Line, count: Int):
    fn at(self, x: Int) -> (val: Int, count: Int) = (val: line.at(x), count: count)

fn cover(fee: Int) -> (val: Int, count: Int):
    var photos = LiChao<Photo, (val: Int, count: Int)>(min, over: 0..=m)
    var best = (val: 0, count: 0)
    for (prev, s) in spans.with_prev:
        let (lo, x) = (Int(s.l), Int(s.r))
        let shared = prev.map(p => p.clip(s).len) ?? 0
        photos.add(Photo(Line(-2 * lo, best.val + lo * lo - shared * shared + fee), best.count + 1))
        let b = photos.at(x)!
        best = (val: b.val + x * x, count: b.count)
    best

print(aliens(k, cover, lambdas: 0..=m * m))
