"""ioi12p1 (Pebbling Odometer) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

The input is the subtask whose odometer program to print (the grids are checker.py's): seed s
gives subtask (s - 1) % 5 + 1, and --max gives subtask 5, whose program runs longest.
"""
import sys

print(5 if sys.argv[1] == "--max" else (int(sys.argv[1]) - 1) % 5 + 1)
