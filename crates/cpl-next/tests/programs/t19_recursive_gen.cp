// expect: ok, output "312802\n9 -1"
// input: "7\n"
// A generator that calls itself: a walk over a tree stored as child lists.
struct Tree:
    kids: List<List<Int>>

    fn walk(self, v: Int) -> yields Int:
        yield v
        for c in kids[v]:
            for x in self.walk(c): yield x
        yield 0 - v

fn find(t: Tree, want: Int) -> Int:
    var steps = 0
    for x in t.walk(0):
        steps += 1
        if x == want: return steps
    0 - 1
let n: Int = read()
var kids: List<List<Int>> = []
for i in 0..n:
    var none: List<Int> = []
    kids.push(none)
for i in 1..n:
    var k = kids[(i - 1) / 2]
    k.push(i)
    kids[(i - 1) / 2] = k
let t = Tree(kids)
var line = 0
for x in t.walk(0):
    line = line * 3 + x
    if x == 0 - 2: break
print(line)
print("{find(t, 5)} {find(t, 100)}")
