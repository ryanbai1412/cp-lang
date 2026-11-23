// `Char` is ASCII, so `Char(i)` stops the program past 127 (R02.2.1).
let i = read<Int>()
print(Char(i - 100), Char(i - 67))
print(Char(i))
