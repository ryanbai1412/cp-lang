// expect: ok, output "3"
domain V = 0..5
var left = 1
while let y = (if left > 0: V.at(3) else: None):
    left -= 1
    print(y)
