#include <bits/stdc++.h>
// egoi23p3: a sentinel step, a per-move bounds test, -1 for none.
static int H, W;
static inline std::pair<int,int> step(std::pair<int,int> p, char m) {
    if (m == '<') return p.second - 1 >= 0 ? std::pair<int,int>{p.first, p.second - 1} : std::pair<int,int>{-1, 0};
    if (m == '>') return p.second + 1 < W ? std::pair<int,int>{p.first, p.second + 1} : std::pair<int,int>{-1, 0};
    if (m == '^') return p.first - 1 >= 0 ? std::pair<int,int>{p.first - 1, p.second} : std::pair<int,int>{-1, 0};
    return p.first + 1 < H ? std::pair<int,int>{p.first + 1, p.second} : std::pair<int,int>{-1, 0};
}
int main(){std::string s;std::cin>>H>>W>>s;long long total=0;
for(int b=0;b<200;b++){std::pair<int,int> box{b%H,b*7%W}, at{0,0};
 for(char m:s){auto nx=step(at,m); if(nx.first!=-1 && nx!=box) at=nx;}
 total+=at.first+at.second;}
printf("%lld\n",total);}
