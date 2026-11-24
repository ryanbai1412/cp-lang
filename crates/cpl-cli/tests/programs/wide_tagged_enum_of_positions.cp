let n: 1..=1e5 = read()
domain V = 1..=n
enum Op = @tag(1) Paint(x: V) | @tag(2) Path(x: V, y: V)
for op in read<[2]Op>():
    match op:
        Paint(x) => print("paint", x)
        Path(x, y) => print("path", x, y)
