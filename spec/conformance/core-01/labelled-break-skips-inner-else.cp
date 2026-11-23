// expect: ok, output "done"
// rule: 01.7.2
'outer: for k in 0..2:
    for j in 0..3:
        if j == 1: break 'outer
    else:
        print("inner else")
print("done")
