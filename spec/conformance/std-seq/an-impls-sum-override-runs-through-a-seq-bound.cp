// expect: ok, output "100 100 100 100 100"
// rule: 08.1.4
// engines: next
// An impl's `sum` overrides the `Seq` default for its type, and generic code with a `Seq` bound
// calls the override, by method, with `of:`, and by the free form `sum(s)` (spec 07, R08.1.4).
struct Hundred(n: Int)
impl Seq<Int> for Hundred:
    fn iter(self) -> yields Int:
        for i in 0..self.n: yield 1
    fn sum<U: Zero + Add>(self, of: Int -> U = x => x) -> U:
        var t = U.zero
        for i in 0..100: t = t + of(1)
        t
fn total<S: Seq<Int>>(s: S) -> Int = s.sum()
fn total_of<S: Seq<Int>>(s: S) -> Int = s.sum(of: x => x)
fn total_free<S: Seq<Int>>(s: S) -> Int = sum(s)
print(Hundred(3).sum(), total(Hundred(3)), total_of(Hundred(3)), total_free(Hundred(3)), sum(Hundred(3)))
