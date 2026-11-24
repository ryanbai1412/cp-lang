"""gen.py SEED SIZE | --max: DMOJ quine. There is no input, so every case, --max included, is
empty (scripts/corpus.py ends it with one newline)."""
import sys
assert sys.argv[1] == "--max" or int(sys.argv[2]) >= 1
