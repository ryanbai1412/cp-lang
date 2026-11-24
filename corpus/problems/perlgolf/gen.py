"""gen.py SEED SIZE | --max: DMOJ perlgolf. The statement allows no more than 0 bytes of input,
so every case, --max included, is empty (scripts/corpus.py ends it with one newline)."""
import sys
assert sys.argv[1] == "--max" or int(sys.argv[2]) >= 1
