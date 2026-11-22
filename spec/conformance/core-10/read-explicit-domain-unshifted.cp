// expect: ok, output "1"
// rule: 10.2.2
// input: "2\n"
domain V = 1..=5
with io(base: 1):
    let v = read<V>()
    print(if v == V(2): 1 else: 0)
