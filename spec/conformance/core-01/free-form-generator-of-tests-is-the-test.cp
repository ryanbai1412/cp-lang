// expect: ok, output "2 3 1"
// rule: 01.10.1
// engines: next
// A generator of tests passed to a `@free_form` member whose next parameter is a test is that
// test: `hits(x > 1 for x in xs)` is `xs.hits(x => x > 1)`, as for a plain fn (decision 323).
trait Hits:
    @free_form
    fn hits(self, p: Int -> Bool) -> Int
impl<S: Seq<Int>> Hits for S:
    fn hits(self, p: Int -> Bool) -> Int:
        var n = 0
        for x in self:
            if p(x): n += 1
        n
let xs = [1, 2, 3]
print(hits(x > 1 for x in xs), hits(x > 0 for x in 0..4), xs.hits(x => x == 2))
