// expect: ok, output "0 1 2\n10 11 11"
// rule: 06.1.6
// One name, several label sets: a call takes the fn whose labels it gives (R06.1.6).
fn pick(xs: List<Int>) -> Int = 0
fn pick(xs: List<Int>; by: Int -> Int) -> Int = 1
fn pick(xs: List<Int>; cmp: (Int, Int) -> Bool) -> Int = 2
fn tag(self: List<Int>) -> Int = 10
fn tag(self: List<Int>; by: Int -> Int) -> Int = 11
print(pick([1]), pick([1], by: x => x), pick([1], cmp: (a, b) => a < b))
print([1].tag(), [1].tag(by: x => x), tag([1], by: x => x))
