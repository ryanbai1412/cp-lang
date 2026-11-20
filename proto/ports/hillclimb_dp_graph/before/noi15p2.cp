// cp: 0.1 0f8d77b+dirty
// NOI 2015 Package Manager: package 0 depends on nothing, every other one on
// its parent. Installing a package installs everything it depends on;
// uninstalling removes everything that depends on it. Print how many
// packages each command changes.
let n: 1..=1e5 = read()
domain Pkg = 0..n
domain Dependent = 1..n

fn random_parents() -> [Dependent]Pkg:
    [Dependent] d => Pkg(rand(0, d.label - 1))

@tags("install", "uninstall")
enum Command:
    Install(pkg: Pkg)
    Uninstall(pkg: Pkg)

let parent: [Dependent]Pkg = read(gen: random_parents)
var needed_by: [Pkg]List<Pkg> = []
for d in Dependent: needed_by[parent[d]].push(Pkg(d.label))
let tree = Rooted(needed_by, Pkg.first)

// Heavy-light layout: visit the largest child first, so a subtree is one
// range of cells and a path to the root is a few.
var cell: [Pkg]Int = 0
var top: [Pkg]Pkg = [Pkg] p => p
var stack: List<Pkg> = [Pkg.first]
var cells = 0
while let u = stack.pop():
    cell[u] = cells
    cells += 1
    var heavy: ?Pkg = none
    for c in tree.children(u):
        if let h = heavy:
            if tree.size(c) > tree.size(h): heavy = c
        else: heavy = c
    for c in tree.children(u):
        if heavy != c: stack.push(c)
    if let h = heavy:
        top[h] = top[u]
        stack.push(h)

// installed packages per segment-tree node, with a pending "set all" below it
domain Node = 1..4 * n
var count: [Node]Int = 0
var pending: [Node]?Int = none

fn put(o: Node, l: Int, r: Int, x: Int):
    count[o] = x * (r - l + 1)
    pending[o] = x

fn push(o: Node, l: Int, r: Int):
    if let x = pending[o]:
        let mid = (l + r) / 2
        put(Node(2 * o.label), l, mid, x)
        put(Node(2 * o.label + 1), mid + 1, r, x)
        pending[o] = none

fn assign(o: Node, l: Int, r: Int, ql: Int, qr: Int, x: Int):
    if ql <= l and r <= qr:
        put(o, l, r, x)
        return
    push(o, l, r)
    let mid = (l + r) / 2
    if ql <= mid: assign(Node(2 * o.label), l, mid, ql, qr, x)
    if qr > mid: assign(Node(2 * o.label + 1), mid + 1, r, ql, qr, x)
    count[o] = count[Node(2 * o.label)] + count[Node(2 * o.label + 1)]

fn total(o: Node, l: Int, r: Int, ql: Int, qr: Int) -> Int:
    if ql <= l and r <= qr: return count[o]
    push(o, l, r)
    let mid = (l + r) / 2
    var s = 0
    if ql <= mid: s += total(Node(2 * o.label), l, mid, ql, qr)
    if qr > mid: s += total(Node(2 * o.label + 1), mid + 1, r, ql, qr)
    s

let q: 1..=1e5 = read()
for _ in 0..q:
    let cmd: Command = read()
    match cmd:
        Install(pkg):
            var fresh = 0
            var u: ?Pkg = pkg
            while let v = u:
                let (l, r) = (cell[top[v]], cell[v])
                fresh += r - l + 1 - total(Node.first, 0, n - 1, l, r)
                assign(Node.first, 0, n - 1, l, r, 1)
                u = tree.parent(top[v])
            print(fresh)
        Uninstall(pkg):
            let (l, r) = (cell[pkg], cell[pkg] + tree.size(pkg) - 1)
            print(total(Node.first, 0, n - 1, l, r))
            assign(Node.first, 0, n - 1, l, r, 0)
