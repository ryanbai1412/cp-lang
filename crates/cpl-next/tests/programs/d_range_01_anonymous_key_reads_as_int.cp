// expect: ok, output "5\n7\n9"
// input: "3\n5 6 7\n"
// rule: R04.3.4
// A key of an array over an anonymous range is a plain `Int` where one is wanted.
let n: 1..=10 = read()
let a: [0..n]Int = read()
for (i, x) in a.items:
    let k: Int = i
    print(k + x)
