// expect: ok, output "YES NO"
// rule: 02.6.7
domain V = 1..=4
enum Dir = L | R | U | D            // payload-free: an index type
enum Ev:
    Add(v: V)
    Ask(l: V, r: V)
    fn is_query(self) -> Bool = self is Ask
print(yes(Ask(1, 2).is_query()), yes(Add(3).is_query()))
