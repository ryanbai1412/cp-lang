// expect: ok, output "0.667\n0.67"
// rule: 10.3.4
print(2.0 / 3.0, precision: 3)
with io(precision: 2):
    print(2.0 / 3.0)
