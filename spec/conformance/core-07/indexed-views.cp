// expect: ok, output "20 1 3"
// rule: 07.1.2 07.2.2 07.2.13 07.3.5 07.4.7 07.4.9 07.5.3 07.5.4
let a = [1, 2, 3]
print(a.map(x => x * 10)[1], a.rev[0], a.rev.len)
