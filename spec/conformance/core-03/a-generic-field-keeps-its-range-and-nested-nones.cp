// expect: ok, output "-994 -7 5 3 -889 7 4 YES YES"
// rule: 03.2.1
// A range given to a generic struct keeps its spare values, so `?F` and
// `?(?F)` fields hold `None` and `Some(None)` apart from every value.
type Small = -1000..=1000
type Tag = ?Small
struct Box<F>:
    x: F
    xs: List<F>
    var os: List<?F>
    var o: ?F
var a = Box<-1000..=1000>(1, [2], [None], None)
var b = Box<Small>(-1000, [1000], [None, Some(-7)], Some(5))
var t = Box<Tag>(None, [Some(3), None], [None, Some(None)], Some(None))
a.xs.push(3)
b.os.push(Some(4))
t.os.push(Some(Some(-1000)))
t.o = Some(Some(7))
var c: List<?Tag> = [None, Some(None), Some(Some(1000))]
c.push(Some(Some(-3)))
var s = 0
for o in t.os:
    s += match o:
        None => 1
        Some(None) => 10
        Some(Some(v)) => 100 + v
print(a.x + b.x + a.xs.len + b.os.len, b.os[1]!, b.o!, t.os.len, s, t.o!!, c.len, yes(c[1] == Some(None)), yes(c[3] == Some(Some(-3))))
