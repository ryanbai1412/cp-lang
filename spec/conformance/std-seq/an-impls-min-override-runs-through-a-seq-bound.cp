// expect: ok, output "-7 -7 -7 8 10 8"
// rule: 08.1.4
// engines: next
// An impl's `min` overrides the `Seq` default for its type, and generic code with a `Seq` bound
// calls the override, with or without `by:`; a type without one gets the default (spec 07,
// R08.1.4).
struct Low(n: Int)
impl Seq<Int> for Low:
    fn iter(self) -> yields Int:
        for i in 0..self.n: yield 10 - i
    fn min<K: Ord>(self, by: Int -> K = x => x, ties: Ties = Ties.First) -> ?Int = Some(-7)
struct Plain(n: Int)
impl Seq<Int> for Plain:
    fn iter(self) -> yields Int:
        for i in 0..self.n: yield 10 - i
fn least<S: Seq<Int>>(s: S) -> Int = s.min() ?? 99
fn least_by<S: Seq<Int>>(s: S) -> Int = s.min(by: x => -x) ?? 99
print(Low(3).min() ?? 99, least(Low(3)), least_by(Low(3)), least(Plain(3)), least_by(Plain(3)), Plain(3).min() ?? 99)
