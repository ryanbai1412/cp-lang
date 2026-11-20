import re,glob,collections
files=glob.glob('../corpus/**/*.cpp',recursive=True)
c=collections.Counter(); ex=collections.defaultdict(list); nf=collections.Counter()
for f in files:
    s=open(f,errors='ignore').read(); seen=set()
    for m in re.finditer(r'struct\s+(\w+)[^{;()]*\{',s):
        name=m.group(1); body=s[m.end():]
        d=1;i=0;top=[]; line=''
        while i<len(body) and d>0:
            ch=body[i]
            if ch=='{': d+=1
            elif ch=='}': d-=1
            elif d==1: line+=ch
            i+=1
        for st in line.split(';'):
            st=st.strip()
            mm=re.match(r'^(const\s+)?([\w:<>, ]+?)\s*([*&])\s*(\w+)$',st)
            if not mm: continue
            ty,k,fld=mm.group(2).strip(),mm.group(3),mm.group(4)
            if ty in('return','else','delete'): continue
            selfref=ty.startswith(name)
            cat='self_ptr' if selfref else ('ref_field' if k=='&' else 'ptr_field')
            c[cat]+=1
            if cat not in seen: nf[cat]+=1; seen.add(cat)
            if len(ex[cat])<6: ex[cat].append(f"{f.split('/')[-1]}:{name}.{fld}:{ty}{k}")
for k,v in c.most_common(): print(f'{k:10} fields={v:4} files={nf[k]:4} ({100*nf[k]/len(files):.1f}%) {ex[k]}')
