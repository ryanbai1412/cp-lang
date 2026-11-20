// Cost of index bounds checks and overflow traps on typical kernels.
#include <bits/stdc++.h>
using namespace std;
#ifndef IDX
#define IDX 0
#endif
#ifndef OVF
#define OVF 0
#endif
[[noreturn]] __attribute__((cold,noinline)) void die(){ fprintf(stderr,"trap\n"); exit(1); }
template<class T> struct Arr {
    T* p; size_t n;
    Arr(size_t n, T v=T()):p(new T[n]),n(n){ fill(p,p+n,v);} 
    inline T& operator[](size_t i){ if (IDX && __builtin_expect(i>=n,0)) die(); return p[i]; }
};
static inline int64_t add(int64_t a,int64_t b){ if(OVF){int64_t r; if(__builtin_add_overflow(a,b,&r)) die(); return r;} return a+b; }
static inline int64_t mul(int64_t a,int64_t b){ if(OVF){int64_t r; if(__builtin_mul_overflow(a,b,&r)) die(); return r;} return a*b; }
static uint64_t rng=88172645463325252ull; static inline uint64_t nxt(){ rng^=rng<<7; rng^=rng>>9; return rng; }
double now(){ return chrono::duration<double>(chrono::steady_clock::now().time_since_epoch()).count(); }

int64_t k_prefix(int n,int q){ Arr<int64_t> a(n), pre(n+1); for(int i=0;i<n;i++) a[i]=nxt()%1000000000;
  for(int i=0;i<n;i++) pre[i+1]=add(pre[i],a[i]); int64_t s=0;
  for(int t=0;t<q;t++){ int l=nxt()%n, r=nxt()%n; if(l>r) swap(l,r); s=add(s, pre[r+1]-pre[l]); s%=1000000007; } return s; }
int64_t k_segtree(int n,int q){ int sz=1; while(sz<n) sz<<=1; Arr<int64_t> t(2*sz,0); int64_t s=0;
  for(int it=0;it<q;it++){ int p=nxt()%n; int64_t v=nxt()%1000000000; p+=sz; t[p]=v; for(p>>=1;p;p>>=1) t[p]=add(t[2*p],t[2*p+1]);
    int l=nxt()%n+sz, r=nxt()%n+sz+1; if(l>=r) continue; int64_t res=0; for(;l<r;l>>=1,r>>=1){ if(l&1) res=add(res,t[l++]); if(r&1) res=add(res,t[--r]); } s^=res; } return s; }
int64_t k_bfs(int n,int m){ Arr<int> head(n+1,0), to(m); vector<pair<int,int>> e(m); for(auto&x:e){x={int(nxt()%n),int(nxt()%n)}; }
  for(auto&x:e) head[x.first+1]++; for(int i=0;i<n;i++) head[i+1]+=head[i]; { Arr<int> pos(n); for(int i=0;i<n;i++) pos[i]=head[i]; for(auto&x:e) to[pos[x.first]++]=x.second; }
  Arr<int> d(n,-1), qu(n); int64_t s=0; for(int src=0;src<4;src++){ for(int i=0;i<n;i++) d[i]=-1; int h=0,tl=0; qu[tl++]=src; d[src]=0;
  while(h<tl){ int u=qu[h++]; for(int k=head[u];k<head[u+1];k++){ int v=to[k]; if(d[v]<0){ d[v]=d[u]+1; qu[tl++]=v; } } } for(int i=0;i<n;i++) s=add(s,d[i]); } return s; }
int64_t k_knap(int n,int W){ Arr<int64_t> dp(W+1,0); for(int i=0;i<n;i++){ int w=nxt()%100+1; int64_t v=nxt()%1000000; for(int c=W;c>=w;c--) dp[c]=max(dp[c], add(dp[c-w],v)); } return dp[W]; }
int64_t k_matmul(int n){ Arr<int64_t> A(n*n),B(n*n),C(n*n,0); for(int i=0;i<n*n;i++){A[i]=nxt()%1000;B[i]=nxt()%1000;}
  for(int i=0;i<n;i++) for(int k=0;k<n;k++){ int64_t a=A[i*n+k]; for(int j=0;j<n;j++) C[i*n+j]=add(C[i*n+j], mul(a,B[k*n+j])); } int64_t s=0; for(int i=0;i<n*n;i++) s^=C[i]; return s; }
int main(){
  auto run=[&](const char* name, auto f){ double best=1e9; int64_t r=0; for(int rep=0;rep<5;rep++){ rng=88172645463325252ull; double t=now(); r=f(); best=min(best,now()-t);} printf("%-8s %8.1f ms  (%lld)\n",name,best*1e3,(long long)(r&0xffff)); };
  run("prefix",[]{return k_prefix(1000000,5000000);});
  run("segtree",[]{return k_segtree(1<<18,1000000);});
  run("bfs",[]{return k_bfs(1000000,4000000);});
  run("knap",[]{return k_knap(2000,100000);});
  run("matmul",[]{return k_matmul(400);});
}
