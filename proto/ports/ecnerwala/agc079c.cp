// AtCoder AGC 079 C: sort a permutation through a deque that may be reversed,
// counting the fewest reversals (or -1). Pass 1 walks the input backwards
// keeping the unplaced values as a stack of linked runs and records, for each
// value, which earlier value it must enter on the same or the opposite side
// of. Pass 2 replays the deque with those choices and counts the reversals.
type Run = (head: Int, tail: Int, neq: Bool)  // neq: must differ from the run below
@!multitest
let n: 1..=2e5 = read()
let perm: [0..n]1..=n = read()
let a = [x - 1 for x in perm]

var pred = List.filled(n, -1)
var succ = List.filled(n, -1)
var runs = List<Run>()
var rel = List.filled(n, (to: -1, flip: false))  // to == -1: free choice

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
            rel[i] = if top.neq: (to: runs.last!.head, flip: true) else: (to: -1, flip: false)
        else:
            runs.last!.head = succ[top.head]
            rel[i] = (to: succ[top.head], flip: false)
        continue
    var l = runs.pop()!
    while let r = runs.last and r.head != a[i]:
        if l.neq:
            print(-1)
            return
        l = swallow(l)
    if runs.len == 0:
        if l.tail != a[i]:
            print(-1)
            return
        l.tail = pred[l.tail]  // a[i] leaves from the bottom of the only run
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
        runs.last!.head = succ[under.head]
        rel[i] = (to: succ[under.head], flip: false)
    runs.push(l)

// side[v]: the value whose side decides v's, and whether v is opposite to it
var side = List.filled(n, (base: 0, flip: false))
for i in 0..n:
    let r = rel[i]
    side[a[i]] = if r.to == -1: (base: a[i], flip: false) else: (
        base: side[r.to].base,
        flip: side[r.to].flip != r.flip,
    )

var front: List<?Bool> = List.filled(n, none)
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
    while q.len > 0 and (a[next] == q.first! or a[next] == q.last!):
        if (if at_front: q.last! else: q.first!) != a[next]:
            at_front = not at_front
            if q.len != 1: turns += 1
        if at_front: q.pop() else: q.pop_front()
        next -= 1
print(turns)
