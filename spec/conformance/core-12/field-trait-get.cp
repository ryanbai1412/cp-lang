// expect: ok, output "3"
// rule: 12.11.3
trait Wt:
    @get fn w(self) -> Int
fn wt<T: Wt>(x: T) -> Int = x.w
print(wt((w: 3, to: 1)))
