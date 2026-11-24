"""helloworldhard: there is no input, so every case (seeded or --max) is empty.

`gen.py SEED SIZE` or `gen.py --max`; the output is "Hello, World!" on three lines.
"""
import sys

assert sys.argv[1] == "--max" or (len(sys.argv) == 3 and int(sys.argv[2]) >= 1)
