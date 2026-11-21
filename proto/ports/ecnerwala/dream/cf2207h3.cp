// Codeforces 2207 H3: Bowser's Castle (Hard Version, interactive).
// Active leaves get strictly increasing values 2*(i+1); deleted min-leaves get
// +INF and max-leaves get 1 (-INF). Querying the array returns the unique
// winning leaf p (leftmost child at min, rightmost at max); probing p = -INF
// and p = +INF finds its lowest max-ancestor sibling a and min-ancestor sibling b.
// One swap of [a, p) and (p, b] separates min vs max parent (<= 3n - 3 queries);
// replaying deletions in reverse rebuilds the canonical alternating min-max tree.

@!interactive
@!multitest
let n: 2..=400 = read()
domain Var = 0..n
const INF: Int = 1_000_000_000

enum Op:
    Min
    Max

enum Expr:
    Leaf(v: Var)
    Node(op: Op, children: List<Expr>)

    fn first_var(self) -> Var = match self:
        Leaf(v) => v
        Node(_, ch) => ch.first.first_var()

    fn last_var(self) -> Var = match self:
        Leaf(v) => v
        Node(_, ch) => ch.last.last_var()

    fn eval(self, x: [Var]Int) -> Int = match self:
        Leaf(v) => x[v]
        Node(Op.Min, ch) => ch.map(c => c.eval(x)).min()!
        Node(Op.Max, ch) => ch.map(c => c.eval(x)).max()!

fn combine(op: Op, items: List<Expr>) -> Expr:
    var out: List<Expr> = []
    for it in items:
        match it:
            Expr.Node(k, ch) if k == op => out.extend(ch)
            _ => out.push(it)
    if out.len == 1: out.first else: Expr.Node(op, out)

fn insert_leaf(t: Expr, v: Var, op: Op) -> Expr:
    if op == Op.Min and t.first_var() > v:
        return combine(Op.Min, [Expr.Leaf(v), t])
    if op == Op.Max and t.last_var() < v:
        return combine(Op.Max, [t, Expr.Leaf(v)])
    let Expr.Node(cur_op, ch) = t else: unreachable()
    if let i = ch.first_index(c => c.first_var() < v < c.last_var()):
        return combine(cur_op, ch[..i] + [insert_leaf(ch[i], v, op)] + ch[i + 1..])
    let split = ch.first_index(c => c.first_var() > v) ?? ch.len
    match (op, cur_op):
        (Op.Min, Op.Min) => combine(Op.Min, ch[..split] + [Expr.Leaf(v)] + ch[split..])
        (Op.Min, Op.Max) => combine(Op.Max, ch[..split] + [combine(Op.Min, [Expr.Leaf(v), combine(Op.Max, ch[split..])])])
        (Op.Max, Op.Max) => combine(Op.Max, ch[..split] + [Expr.Leaf(v)] + ch[split..])
        (Op.Max, Op.Min) => combine(Op.Min, [combine(Op.Max, [combine(Op.Min, ch[..split]), Expr.Leaf(v)])] + ch[split..])

enum Slot:
    Active
    DropMin
    DropMax

var state: [Var]Slot = Slot.Active

fn base_val(v: Var) -> Int = match state[v]:
    Slot.Active => 2 * (v.ord + 1)
    Slot.DropMax => 1
    Slot.DropMin => INF

fn ask(vals: [Var]Int) -> Int:
    print("?", vals)
    read<Int>()

fn ask_winner(override: ?(Var, Int) = none) -> ?Var:
    var vals: [Var]Int = [Var] v => base_val(v)
    if let (u, val) = override:
        vals[u] = val
    let ans = ask(vals)
    if ans == 1 or ans == INF: none else: Var.nth(ans / 2 - 1)

var p = ask_winner()!
var deletions: List<(Var, Op)> = []

repeat n - 1:
    let a = ask_winner((p, 1))
    let b = ask_winner((p, INF))
    let parent_op = match (a, b):
        (none, _) => Op.Min
        (_, none) => Op.Max
        (some(la), some(rb)) =>
            let hi = 2 * (rb.ord + 1) + 1
            let lo = 2 * (la.ord + 1) - 1
            var probe: [Var]Int = [Var] v =>
                if state[v] != Slot.Active: base_val(v)
                elif la <= v < p: hi
                elif p < v <= rb: lo
                else: base_val(v)
            if ask(probe) == hi: Op.Min else: Op.Max
    deletions.push((p, parent_op))
    if parent_op == Op.Min:
        state[p] = Slot.DropMin
        p = b!
    else:
        state[p] = Slot.DropMax
        p = a!

var tree = Expr.Leaf(p)
for (v, op) in deletions.rev:
    tree = insert_leaf(tree, v, op)

print("!")
loop:
    let first = read<Int>()
    if first == 0: break
    var x: [Var]Int = 0
    x[Var.first] = first
    for v in Var.first + 1..=Var.last:
        x[v] = read()
    print(tree.eval(x))
