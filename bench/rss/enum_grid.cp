// cco20p4's colour grid: an n x n table of a two-variant enum, read at random. Each cell is a
// byte, as in the matched C++.
enum Colour: Red | Blue
let n: 1..=4000 = read()
var grid = List.filled(n * n, Colour.Red)
var x = 12345
for i in 0..n * n:
    x = (x * 1103515245 + 12345) % 2147483647
    if x % 3 == 0: grid[i] = Colour.Blue
var blue = 0
for i in 0..n * n:
    x = (x * 1103515245 + 12345) % 2147483647
    if grid[x % (n * n)] == Colour.Blue: blue += 1
print(blue)
