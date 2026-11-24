import sys
# output-only: the input is a task number; seed s picks task s mod 10 + 1 (size is ignored), and
# --max asks for task 10, the largest computer (64 rounds of shift-and-subtract)
print(10 if sys.argv[1] == "--max" else int(sys.argv[1]) % 10 + 1)
