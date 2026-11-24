// expect: ok
// rule: 02.7.1
// tool: fix
// rewritten: "domain V = 0..5\nlet t = gen<Rooted<V>>(shape: Path)\nprint(V.count)"
// `shape: path` from before `TreeShape` gets the variant's spelling from the enum's declaration.
domain V = 0..5
let t = gen<Rooted<V>>(shape: path)
print(V.count)
