// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
var left = 1
while let y = (if left > 0: xs else: None):
    left -= 1
    print(y)
