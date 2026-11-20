// NOI 2004 Cashier: a payroll under four commands. "I x" hires someone at
// salary x, who walks straight out if x is below the wage floor. "A x"
// raises every salary by x and "S x" cuts every salary by x, after which
// everyone below the floor leaves. "F k" asks for the k-th highest salary,
// or -1 if there are fewer than k staff. At the end print how many people
// left after a cut.
//
// A raise or a cut moves everyone together, so store each salary without
// the raises given since: pay = base + raise, and the order of the bases
// never changes. The original keeps them in an order-statistics tree. Here
// the commands are read twice: once to learn every base that will occur,
// once to count staff per base in a Fenwick tree.

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

let bases = sorted(seen)
domain Slot = 0..bases.len
var slot = Map<Int, Slot>()
for s in Slot: slot[bases[s.to_idx]] = s

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
            let lowest = staff.lower_bound(1)!.left!
            if bases[lowest.to_idx] + raise >= floor: break
            let leaving = staff.sum(lowest..=lowest)
            staff.add(lowest, -leaving)
            size -= leaving
            left += leaving
    elif x > size:
        print(-1)
    else:
        let kth = staff.lower_bound(size - x + 1)!.left!
        print(bases[kth.to_idx] + raise)
print(left)
