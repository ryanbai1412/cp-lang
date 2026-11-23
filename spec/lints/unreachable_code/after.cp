fn first_even(xs: List<Int>) -> Int:
    for x in xs:
        if x % 2 == 0:
            return x
    return -1
print(first_even([3, 4, 5]))
