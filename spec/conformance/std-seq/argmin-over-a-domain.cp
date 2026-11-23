// expect: ok, output "2 0 4"
// rule: std.seq argmin
// `argmin` and `argmax` walk a domain directly and answer with one of its elements.
domain I = 0..5
let a: [I]Int = [I] i => (i.ord - 2) * (i.ord - 2)
print(I.argmin(i => a[i])!, I.argmax(i => a[i])!, I.argmax(i => i.ord)!)
