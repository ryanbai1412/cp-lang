// expect: E0501
// rule: 08.2.6 12.11.1
trait Sh:
    fn area(self) -> Int
trait Both = Sh + Ord
struct Sq: s: Int
impl Both for Sq // <- here
