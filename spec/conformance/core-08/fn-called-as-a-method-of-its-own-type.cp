// expect: ok, output "1"
// rule: 08.1.4
// A module's fn called as a method on its own first parameter's type hides nothing, even where
// its body calls the catalog member of the same name on a List.
struct Ring(xs: List<Int>)
fn first_gt(self: Ring, t: Int) -> ?Int = self.xs.first_gt(t)
print(Ring([3, 5, 8]).first_gt(4) ?? 0)
