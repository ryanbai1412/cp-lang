// expect: ok, output "100 100 100 100 8 8"
// rule: 08.1.4
// engines: next
// An impl's `prod` overrides the `Seq` default for its type, and generic code with a `Seq` bound
// calls the override by method and by the free form `prod(s)`; a type without one gets the
// default (spec 07, R08.1.4).
struct Hundred(n: Int)
impl Seq<Int> for Hundred:
    fn iter(self) -> yields Int:
        for i in 0..self.n: yield 2
    fn prod(self) -> Int: 100
struct Twos(n: Int)
impl Seq<Int> for Twos:
    fn iter(self) -> yields Int:
        for i in 0..self.n: yield 2
fn total<S: Seq<Int>>(s: S) -> Int = s.prod()
fn total_free<S: Seq<Int>>(s: S) -> Int = prod(s)
print(Hundred(3).prod(), total(Hundred(3)), total_free(Hundred(3)), prod(Hundred(3)), total(Twos(3)), prod(Twos(3)))
