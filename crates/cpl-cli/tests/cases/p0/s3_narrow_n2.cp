// Synthesis #3 Leap 2 (Rule N2 + Invariant 4): syntactic non-empty narrowing on List and user `@law some_when`.
struct MyStack:
    items: List<Int>

    @get
    fn is_empty(self) -> Bool = self.items.is_empty

    @law some_when(not self.is_empty)
    fn pop(mut self) -> ?Int = self.items.pop()

let n: 1..=10 = read()
var q: List<Int> = []
for i in 1..=n: q.push(i * 10)

// if q.len > 0 narrows `.first` and `.last`
var ends = 0
if q.len > 0:
    ends = q.first + q.last

// short-circuit N2
let big = not q.is_empty and q.last >= 40

// while not q.is_empty narrows `q.pop()` (single pop per guard; mutation invalidates afterwards)
var total = 0
while not q.is_empty:
    let x: Int = q.pop()
    total += x

// Invariant 4: redundant `!` inside `while not q2.is_empty:` remains valid
var q2: List<Int> = [1, 2, 3]
var s2 = 0
while not q2.is_empty:
    s2 += q2.pop()!

// Userland `@law some_when(not self.is_empty)` struct
var st = MyStack(items: [5, 7, 9])
var s3 = 0
while not st.is_empty:
    let top: Int = st.pop()
    s3 += top

print(ends, Int(big), total, s2, s3)
