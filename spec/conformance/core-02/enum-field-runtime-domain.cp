// expect: ok, output "3 0"
// rule: 02.7.2 10.2.3
// input: "3\nUpdate 2 7\n"
// A top-level enum's payload may be a domain sized at run time; the command reads as one value.
let n = read<1..=5>()
domain P = 1..=n
enum Cmd = Update(k: P, u: Int) | Ask(k: P)
let c = read<Cmd>()
print(n, if c is Ask(_): 1 else: 0)
