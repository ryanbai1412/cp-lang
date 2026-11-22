// expect: ok, output "1 2"
// rule: 01.7.2
'outer: for r in 1..=3:
    for c in 1..=3:
        if r + c == 3:
            print(r, c)
            break 'outer
