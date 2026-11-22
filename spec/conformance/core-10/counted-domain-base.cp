// expect: ok, output "0 1 4"
// rule: 10.2.2 10.3.2
with io(base: 1)
let n = 4
domain E = [n]
let e = "1".parse<E>()!
print(Int(e), e, n)
