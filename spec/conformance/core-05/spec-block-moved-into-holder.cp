// expect: E0608 at 4
// rule: 05.3.5
struct Holder: xs: List<Int>
var a = List(1, 2)
let h = Holder(a)          // a moves into h
print(a.len)               // error: a was moved into h on line 3; write Holder(a.clone())
