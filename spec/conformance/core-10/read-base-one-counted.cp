// expect: ok, output "1"
// rule: 10.2.2
// input: "3\n2\n"
let n: 1..=10 = read()
domain V = [n]
with io(base: 1):
    let v = read<V>()
    print(if v == V(1): 1 else: 0)
