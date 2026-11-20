#!/usr/bin/env python3

line = 1


def append(s):
    global line
    print(s)
    line += 1
    return line - 1


def read_input():
    return append("I")


def output(n):
    return append(f"O {n}")


def add(a, b):
    return append(f"+ {a} {b}")


def add_c(a, c):
    return append(f"C {a} {c}")


def negate(a):
    return append(f"- {a}")


def lshift(a, k):
    return append(f"< {a} {k}")


def rshift(a, k):
    return append(f"> {a} {k}")


def sigmoid(a):
    return append(f"S {a}")


def sub(a, b):
    return add(a, negate(b))


def digits(val):
    global line
    print(f"< {val} 3100")  # line
    val = line
    line += 1
    ans = []
    for i in range(31, 0, -1):
        print(f"C {val} {-(2 ** (3100 + i)) + 1000}")  # line
        print(f"S {line}")  # line + 1
        print(f"- {line + 1}")  # line + 2
        print(f"< {line + 2} {3100 + i}")  # line + 3
        print(f"+ {val} {line + 3}")  # line + 4
        ans.append(line + 1)
        val = line + 4
        line += 5
    print(f"> {val} 3100")  # line
    ans.append(line)
    line += 1
    return ans


def st6():
    global line
    print("I")
    line += 1
    d = digits(1)
    for x in d:
        print(f"O {x}")


def st7():
    global line
    print("I")
    print("I")
    line += 2
    ad = digits(1)
    bd = digits(2)
    outd = []
    for a, b in zip(ad, bd):
        print(f"+ {a} {b}")  # line
        print(f"C {line} -1.5")  # line + 1
        print(f"< {line + 1} 3200")  # line + 2
        print(f"S {line + 2}")  # line + 3
        print(f"- {line + 3}")  # line + 4
        print(f"< {line + 4} 1")  # line + 5
        print(f"+ {line} {line + 5}")  # line + 6
        outd.append(line + 6)
        line += 7
    val = outd.pop(0)
    for d in outd:
        print(f"< {val} 1")  # line
        print(f"+ {line} {d}")  # line + 1
        val = line + 1
        line += 2
    print(f"O {val}")


# a if b < 0 else 0
def f1(a, b):
    EPS = "0.000000000000000000000000000000001"
    tmp1 = lshift(sigmoid(lshift(add_c(b, EPS), 1000)), 201)
    tmp2 = lshift(add_c(sigmoid(add(rshift(a, 200), tmp1)), -0.5), 202)
    return sub(tmp2, tmp1)


def sort_2(a, b):
    diff = sub(a, b)
    mn = add(f1(diff, diff), b)
    mx = sub(add(a, b), mn)
    return mn, mx
    # x = add_c(sub(a, b), )
    # tmp1 = lshift(sigmoid(lshift(x, 1000)), 201)
    # tmp2 = lshift(add_c(sigmoid(add(rshift(x, 200), tmp1)), -0.5), 202)
    # mn = add(sub(tmp2, tmp1), b)
    # mx = sub(add(a, b), mn)
    # return mn, mx


def st9():
    global line
    arr = [read_input() for _ in range(16)]
    for i in range(15, -1, -1):
        for j in range(i):
            arr[j], arr[j + 1] = sort_2(arr[j], arr[j + 1])
    for x in arr:
        output(x)


def st8():
    output(lshift(add_c(sigmoid(add_c(rshift(read_input(), 200),
                                      "-2.063437068895560546727281172620131871456591449883392499836032692765902842847409911780353006")),
                        "-0.112701665379258311482073460021760038916707829470840917341242623388651690806302096648071262"),
                  200))


def main(ans):
    t = int(input())

    if t == 6:
        st6()
    elif t == 7:
        st7()
    elif t == 8:
        st8()
    elif t == 9:
        st9()
    else:
        print(ans[t - 1])

ans = ["""I
I
+ 1 2
- 3
+ 4 4
O 5
""", """I
< 1 4
+ 1 2
- 3
S 4
O 5
""", """I
< 1 3200
S 2
C 3 -0.5
+ 4 4
O 5
""", """I
C 1 0.000000000000000000000000000000001
< 2 3200
S 3
< 4 202
> 2 200
+ 5 6
S 7
C 8 -0.5
< 9 203
- 10
+ 5 11
+ 2 12
O 13
""", """I
I
< 1 1
+ 2 3
I
< 4 1
+ 5 6
I
< 7 1
+ 8 9
I
< 10 1
+ 11 12
I
< 13 1
+ 14 15
I
< 16 1
+ 17 18
I
< 19 1
+ 20 21
I
< 22 1
+ 23 24
I
< 25 1
+ 26 27
I
< 28 1
+ 29 30
I
< 31 1
+ 32 33
I
< 34 1
+ 35 36
I
< 37 1
+ 38 39
I
< 40 1
+ 41 42
I
< 43 1
+ 44 45
I
< 46 1
+ 47 48
I
< 49 1
+ 50 51
I
< 52 1
+ 53 54
I
< 55 1
+ 56 57
I
< 58 1
+ 59 60
I
< 61 1
+ 62 63
I
< 64 1
+ 65 66
I
< 67 1
+ 68 69
I
< 70 1
+ 71 72
I
< 73 1
+ 74 75
I
< 76 1
+ 77 78
I
< 79 1
+ 80 81
I
< 82 1
+ 83 84
I
< 85 1
+ 86 87
I
< 88 1
+ 89 90
I
< 91 1
+ 92 93
O 94
""", """""", """""", """""", """""", """"""]
main(ans)