// expect: ok, output "6 3 6 4"
// rule: 07.1.6 07.2.5
domain V = 1..=4
let a = [V] v => Int(v)
let pre = a.scan(+)
let suf = a.rev.scan(+)
let v = V(3)
print(pre[v], pre[v.l], pre[v.r], suf[v.r])
