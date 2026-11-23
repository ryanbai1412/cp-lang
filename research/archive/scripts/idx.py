import re,glob,collections
files=glob.glob('../corpus/**/*.cpp',recursive=True)
names=collections.Counter(); cat=collections.Counter(); fcat=collections.defaultdict(set)
def classify(arr):
    a=arr.lower()
    if re.match(r'(pre|pref|psum|ps|sum|cum|acc|s)\w*$',a): return 'prefix (bounds)'
    if re.match(r'(dp|f|g|memo|best|cnt)\w*$',a): return 'dp over positions'
    if re.match(r'(a|arr|v|b|c|h|p|x|y|s|t|str|w)$',a): return 'neighbour in input'
    return 'other'
for f in files:
    s=open(f,errors='ignore').read()
    for m in re.finditer(r'(\w+)\s*\[\s*(\w+)\s*([+-])\s*1\s*\]',s):
        arr=m.group(1); names[arr]+=1; k=classify(arr); cat[k]+=1; fcat[k].add(f)
    for m in re.finditer(r'\w+\[\s*(tin|tout|pos|id|idx|ord|comp|rk|rnk|order)\s*\[',s):
        cat['index through a map array (derived domain)']+=1; fcat['index through a map array (derived domain)'].add(f)
    for m in re.finditer(r'\[\s*\w+\s*\+\s*d[xyr]\w*\s*\[',s):
        cat['grid step (dx/dy)']+=1; fcat['grid step (dx/dy)'].add(f)
tot=len(files)
for k,v in cat.most_common(): print(f'{k:45} occ={v:5} files={len(fcat[k]):4} ({100*len(fcat[k])/tot:.1f}%)')
print('top arrays with +-1:',names.most_common(25))
