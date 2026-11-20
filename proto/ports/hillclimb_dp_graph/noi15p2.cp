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

// one cell per package, laid out so that a subtree is one range of cells
// and a path to the root is a few
var installed = AssignSum<tree.Pre>()
let q: 1..=1e5 = read()
for _ in 0..q:
    let cmd: Command = read()
    match cmd:
        Install(pkg):
            var fresh = 0
            for (l, r) in tree.chains(Pkg.first, pkg):
                fresh += r.idx - l.idx + 1 - installed.sum(l, r)
                installed.assign(l, r, 1)
            print(fresh)
        Uninstall(pkg):
            let (l, r) = tree.sub(pkg)
            print(installed.sum(l, r))
            installed.assign(l, r, 0)
