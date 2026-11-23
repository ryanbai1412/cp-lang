// expect: ok, output "1 0"
// rule: 02.7.1
// The chapter's `Ev`: a method in the enum body, and `self is Ask` names a payload variant without its fields.
enum Ev:
    Add(v: Int)
    Ask(l: Int, r: Int)
    fn is_query(self) -> Bool = self is Ask
let e = Ask(1, 2)
let f = Add(5)
print(if e.is_query(): 1 else: 0, if f.is_query(): 1 else: 0)
