// expect: ok, output "4\n1\n7"
// rule: 07.4.17
struct Win<S>(src: S, lo: Int, hi: Int)
impl<S: Indexed<K, T>, K, T> Seq<T> for Win<S>:
    fn iter(self) -> yields T:
        for i in self.lo..self.hi: yield self.src.nth(i)
let a = [1, 2, 3, 4, 5, 7]
print(a.take_last(3).first!)
print(a.rebase.first!)
print(Win(a, 5, 6).first!)
