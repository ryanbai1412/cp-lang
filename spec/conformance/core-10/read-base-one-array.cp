// expect: ok, output "3 1 2"
// rule: 10.2.2
// input: "3\n3 1 2\n"
let n: 1..=10 = read()
domain V = [n]
with io(base: 1):
    let p: [n]V = read()
    print(p[0], p[1], p[2])
