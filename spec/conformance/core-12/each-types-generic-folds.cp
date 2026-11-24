// expect: ok, output "bb bb 3"
// rule: 12.4
// A struct whose `each` yields its elements gives a generic fold those elements, not the struct's own type (R12.4.1).
struct W(n: Int):
    pub fn each(self) -> yields Str:
        yield "a"
        yield "bb"
let w = W(1)
print(w.max(by: s => s.len)!, max(w, by: s => s.len)!, w.sum(of: s => s.len))
