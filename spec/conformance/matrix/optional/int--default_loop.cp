// expect: ok, output "5"
let x: ?Int = 5
for i in 0..1:
    print(x ?? 0)
