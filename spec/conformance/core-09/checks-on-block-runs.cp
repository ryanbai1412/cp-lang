// expect: ok, output "3"
// rule: 09.4.1
// input: "2\n"
let n = read<Int>()
with checks(overflow: on):
    print(n + 1)
