// expect: ok
// rule: judge.9
// tool: build --judge generic -o -
// has: "#pragma GCC target(\"popcnt\")"
// lacks: "avx2"
// lacks: "O3"
// The default instruction set, x86-64, asks for popcnt alone: every x86-64 judge CPU has it.
let n: 1..=100 = read()
print(n * 2)
