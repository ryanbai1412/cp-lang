import sys
out=open(sys.argv[2]).read().split(); ans=open(sys.argv[3]).read().split()
assert len(out)==len(ans), "len"
for a,b in zip(out,ans):
    a=float(a); b=float(b); assert abs(a-b)/max(1,abs(b))<=1e-9, (a,b)
