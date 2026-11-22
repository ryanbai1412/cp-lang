// expect: ok, output "5"
let x: ?Int = 5
var left = 1
while let y = (if left > 0: x else: None):
    left -= 1
    print(y)
