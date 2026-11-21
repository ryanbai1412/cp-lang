// AtCoder AGC 079 C: sort a permutation through a deque that may be reversed,
// counting the fewest reversals (or -1). Pass 1 walks the input backwards
// keeping the unplaced values as a stack of linked runs and records, for each
// value, which earlier value it must enter on the same or the opposite side
// of. Pass 2 replays the deque with those choices and counts the reversals.
type Run = (head: Int, tail: Int, neq: Bool)  // neq: must differ from the run below

@!multitest
let n: 1..=2e5 = read()
let perm: [0..n]1..=n = read()
let a = [0..n] i => perm[i] - 1

var pred: [0..n]?Int = none
var succ: [0..n]?Int = none
var runs: List<Run> = []
var rel: [0..n]?(to: Int, flip: Bool) = none

// Append the run under the top of the stack to `l`, taking it off the stack.
fn swallow(l: Run) -> Run:
    let under = runs.pop()!
    succ[l.tail] = under.head
    pred[under.head] = l.tail
    (head: l.head, tail: under.tail, neq: under.neq)

var rem = n
for i in (0..n).rev:
    while a[i] < rem:
        rem -= 1
        runs.push((head: rem, tail: rem, neq: false))
    let top = runs.last!
    if top.head == a[i]:
        if top.tail == top.head:
            runs.pop()
            if top.neq: rel[i] = (to: runs.last!.head, flip: true)
        else:
            let nxt = succ[top.head]!
            runs.last!.head = nxt
            rel[i] = (to: nxt, flip: false)
        continue
    var l = runs.pop()!
    while let r = runs.last, r.head != a[i]:
        if l.neq:
            print(-1)
            return
        l = swallow(l)
    if runs.is_empty:
        if l.tail != a[i]:
            print(-1)
            return
        l.tail = pred[l.tail]!  // a[i] leaves from the bottom of the only run
        rel[i] = (to: l.tail, flip: false)
        runs.push(l)
        continue
    let under = runs.last!
    l.neq = true
    if under.head == under.tail:
        rel[i] = (to: l.head, flip: true)
        runs.pop()
        if under.neq: l = swallow(l)
        else: l.neq = false
    else:
        let nxt = succ[under.head]!
        runs.last!.head = nxt
        rel[i] = (to: nxt, flip: false)
    runs.push(l)

// side[v]: the value whose side decides v's, and whether v is opposite to it
var side: [0..n](base: Int, flip: Bool) = (base: 0, flip: false)
for i in 0..n:
    side[a[i]] = if let r = rel[i]: (
        base: side[r.to].base,
        flip: side[r.to].flip != r.flip,
    ) else: (base: a[i], flip: false)

var front: [0..n]?Bool = none
var at_front = false
var q = Deque<Int>()
var (next, turns) = (n - 1, 0)
for i in (0..n).rev:
    let s = side[i]
    front[s.base] ??= at_front != s.flip
    if at_front != (front[s.base]! != s.flip):
        at_front = not at_front
        if q.len != 1: turns += 1
    if at_front: q.push_front(i) else: q.push(i)
    while let lo = q.first, a[next] == lo or a[next] == q.last!:
        if (if at_front: q.last! else: lo) != a[next]:
            at_front = not at_front
            if q.len != 1: turns += 1
        if at_front: q.pop() else: q.pop_front()
        next -= 1
print(turns)
