// expect: ok
// rule: 11.4.3
// tool: migrate
// rewritten: "print(V.cuts)"
// lacks: "error"
// `cpl migrate` rewrites a removed spelling to its replacement in the file (R11.4.3, tools/cli.md), so an old solution keeps its meaning.
domain V = 0..3
print(V.bounds)
