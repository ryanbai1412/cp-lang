// expect: ok, output "2 2 4 1 3 2 4"
// shifted, complement, supersets and a for over items work on a Subset a program builds and passes through its own fn.
domain B = 0..4
fn grow(s: Subset<B>) -> Subset<B> = s.with(B(3))
let s = grow(Subset<B>.single(B(1)))
var walked: List<B> = []
for b in s.items: walked.push(b)
print(s.shifted(-1).len, s.complement().len, s.supersets.count(), walked[0], walked[1], grow(Subset<B>.empty).len + 1, Subset<B>.full.len)
