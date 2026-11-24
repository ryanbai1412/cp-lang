// expect: E0203
// tool: check
// has: "`partition_point` is spelled `bsearch`"
// std's `@aka(partition_point)` on `bsearch` names any type's own `bsearch`, a user's too.
struct Line:
    xs: List<Int>
    pub fn bsearch(self, ok: Int -> Bool) -> Int = self.xs.bsearch(ok).r
let l = Line([1, 3, 5])
print(l.partition_point(x => x < 3)) // <- here
