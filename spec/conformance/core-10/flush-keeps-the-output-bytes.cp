// expect: ok, output "1\n2 3"
// rule: 10.4.1
// A flush between prints hands the first line over early and changes no byte of the output.
print(1)
flush()
print(2, 3)
flush()
