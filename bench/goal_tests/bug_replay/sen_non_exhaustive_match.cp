enum Dir:
    Up
    Down
    Left
let d = Dir.Up
let x = match d:
    Dir.Up => 1
    Dir.Down => 2
print(x)
