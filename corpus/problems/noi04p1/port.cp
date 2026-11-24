// Source: NOI 2004 P1 (The Depressed Cashier), https://dmoj.ca/problem/noi04p1
// limits: 600 ms, 64 MB
// Problem: n <= 3e5 commands and a wage floor. "I x" hires at salary x (gone at once, and
//   not counted, if below the floor); "A x" raises every salary by x; "S x" cuts every
//   salary by x, then everyone below the floor leaves; "F k" prints the k-th highest salary
//   or -1. At most 100 A/S commands. Finally print how many people left.
// Solution: everyone moves together, so store base = salary - (raise so far); the bases'
//   order never changes. Read the commands twice: first to collect every base that will
//   occur, then count staff per base in a Fenwick tree over the sorted bases. A cut drops
//   the lowest slots while below the floor; "F k" is a Fenwick descent to the
//   (size - k + 1)-th smallest. O(n log n).

let (q, floor): (Int, Int) = read()
domain Cmd = 0..q
let cmds: [Cmd](op: Char, x: Int) = read()

var raise = 0
var seen = Set<Int>()
for c in Cmd:
    let (op, x) = cmds[c]
    if op == 'A': raise += x
    elif op == 'S': raise -= x
    elif op == 'I' and x >= floor: seen.add(x - raise)

let bases = seen.sorted()
domain Slot = 0..bases.len
var slot = Map<Int, Slot>()
for s in Slot: slot[bases[s.ord]] = s

var staff = Fenwick<Slot, Int>()
var (size, left) = (0, 0)
raise = 0
for c in Cmd:
    let (op, x) = cmds[c]
    if op == 'I':
        if x >= floor:
            staff.add(slot[x - raise], 1)
            size += 1
    elif op == 'A':
        raise += x
    elif op == 'S':
        raise -= x
        while size > 0:
            let lowest = staff.first_ge(1)!.left!
            if bases[lowest.ord] + raise >= floor: break
            let leaving = staff.fold(lowest..=lowest)
            staff.add(lowest, -leaving)
            size -= leaving
            left += leaving
    elif x > size:
        print(-1)
    else:
        let kth = staff.first_ge(size - x + 1)!.left!
        print(bases[kth.ord] + raise)
print(left)
