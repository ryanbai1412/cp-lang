// expect: ok, output "none"
// rule: 01.7.2
for x in [1, 2]:
    if x > 5: break
else:
    print("none")
