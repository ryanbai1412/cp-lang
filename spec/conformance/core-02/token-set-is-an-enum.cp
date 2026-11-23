// expect: ok, output "-1 -1"
// input: "UL\n"
// rule: 02.7.2
// A token set from the statement is a payload-free enum; each variant reads as its name.
enum D = DR | DL | UR | UL
let (di, dj) = match read<D>():
    DR => (1, 1)
    DL => (1, -1)
    UR => (-1, 1)
    UL => (-1, -1)
print(di, dj)
