// expect: ok
// rule: judge.9
// tool: build --judge generic -o -
// lacks: "#pragma GCC target"
// lacks: "O3"
// The default instruction set, x86-64, asks for no target pragma.
let n: 1..=100 = read()
print(n * 2)
