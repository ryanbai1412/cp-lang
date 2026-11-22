// expect: ok, output "0 2 -1"
print("ab".search("") ?? -1, "ab".rsearch("") ?? -1, "".search("a") ?? -1)
