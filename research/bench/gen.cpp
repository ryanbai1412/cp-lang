// Generator lowering strategies vs hand-written loops.
#include <bits/stdc++.h>
using namespace std;
static uint64_t rng=88172645463325252ull; static inline uint64_t nxt(){ rng^=rng<<7; rng^=rng>>9; return rng; }
double now(){ return chrono::duration<double>(chrono::steady_clock::now().time_since_epoch()).count(); }
const int N=20000000;
vector<int> a,b;

// 1. runs(a): sum over blocks of len*(len+1)/2
__attribute__((noinline)) int64_t runs_hand(){ int64_t t=0; int n=a.size(); for(int i=0;i<n;){ int j=i; while(j+1<n&&a[j+1]==a[i]) j++; int64_t L=j-i+1; t+=L*(L+1)/2; i=j+1;} return t; }
template<class F> inline __attribute__((always_inline)) void runs_push(const vector<int>& v, F&& yield){ int n=v.size(); for(int i=0;i<n;){ int j=i; while(j+1<n&&v[j+1]==v[i]) j++; yield(i,j+1,v[i]); i=j+1;} }
__attribute__((noinline)) int64_t runs_push_use(){ int64_t t=0; runs_push(a,[&](int l,int r,int){ int64_t L=r-l; t+=L*(L+1)/2;}); return t; }
struct RunsPull{ const vector<int>& v; int i=0; struct R{int l,r,x;}; inline bool next(R& out){ int n=v.size(); if(i>=n) return false; int j=i; while(j+1<n&&v[j+1]==v[i]) j++; out={i,j+1,v[i]}; i=j+1; return true; } };
__attribute__((noinline)) int64_t runs_pull_use(){ int64_t t=0; RunsPull g{a}; RunsPull::R r; while(g.next(r)){ int64_t L=r.r-r.l; t+=L*(L+1)/2;} return t; }

// 2. zip(filter(a, even), map(b, x*3)): dot product of first k pairs
__attribute__((noinline)) int64_t zip_hand(){ int64_t t=0; int n=a.size(),i=0,j=0; while(true){ while(i<n&&(a[i]&1)) i++; if(i>=n||j>=n) break; t+=int64_t(a[i])*(b[j]*3); i++; j++; } return t; }
struct FilterEven{ const vector<int>& v; int i=0; inline bool next(int& x){ int n=v.size(); while(i<n&&(v[i]&1)) i++; if(i>=n) return false; x=v[i++]; return true; } };
struct Map3{ const vector<int>& v; int i=0; inline bool next(int& x){ if(i>=(int)v.size()) return false; x=v[i++]*3; return true; } };
__attribute__((noinline)) int64_t zip_pull(){ int64_t t=0; FilterEven f{a}; Map3 m{b}; int x,y; while(f.next(x)&&m.next(y)) t+=int64_t(x)*y; return t; }
// zip with left side push, right side pull (what "inline the outer, state-machine the inner" gives)
__attribute__((noinline)) int64_t zip_mixed(){ int64_t t=0; Map3 m{b}; int y; bool done=false; int n=a.size(); for(int i=0;i<n&&!done;i++){ if(a[i]&1) continue; if(!m.next(y)){done=true;break;} t+=int64_t(a[i])*y; } return t; }

// 3. tree preorder sum of depth*val on a random tree (children CSR)
int TN=4000000; vector<int> head,ch,val;
int64_t acc; 
void dfs_hand(int u,int d){ acc+=int64_t(d)*val[u]; for(int k=head[u];k<head[u+1];k++) dfs_hand(ch[k],d+1); }
template<class F> void pre_push(int u,int d,F& y){ y(u,d); for(int k=head[u];k<head[u+1];k++) pre_push(ch[k],d+1,y); }
__attribute__((noinline)) int64_t pre_push_use(){ int64_t t=0; auto y=[&](int u,int d){ t+=int64_t(d)*val[u];}; pre_push(0,0,y); return t; }
__attribute__((noinline)) int64_t pre_stack_hand(){ int64_t t=0; vector<pair<int,int>> st; st.reserve(TN); st.push_back({0,0}); while(!st.empty()){ auto [u,d]=st.back(); st.pop_back(); t+=int64_t(d)*val[u]; for(int k=head[u+1]-1;k>=head[u];k--) st.push_back({ch[k],d+1}); } return t; }
struct PrePull{ vector<pair<int,int>> st; PrePull(){ st.reserve(TN); st.push_back({0,0}); } inline bool next(int& u,int& d){ if(st.empty()) return false; tie(u,d)=st.back(); st.pop_back(); for(int k=head[u+1]-1;k>=head[u];k--) st.push_back({ch[k],d+1}); return true; } };
__attribute__((noinline)) int64_t pre_pull_use(){ int64_t t=0; PrePull g; int u,d; while(g.next(u,d)) t+=int64_t(d)*val[u]; return t; }

int main(){
  a.resize(N); b.resize(N); for(int i=0;i<N;i++){ a[i]=nxt()%3; b[i]=nxt()%1000; }
  // random tree: parent of i is random in [max(0,i-50), i) for depth, build CSR
  vector<int> par(TN,-1); for(int i=1;i<TN;i++){ int lo=max(0,i-1000); par[i]=lo+nxt()%(i-lo);} head.assign(TN+1,0); for(int i=1;i<TN;i++) head[par[i]+1]++; for(int i=0;i<TN;i++) head[i+1]+=head[i]; ch.resize(TN); { vector<int> pos(head.begin(),head.end()-1); for(int i=1;i<TN;i++) ch[pos[par[i]]++]=i; } val.resize(TN); for(auto&x:val) x=nxt()%100;
  auto run=[&](const char* nm, auto f){ double best=1e9; int64_t r=0; for(int k=0;k<5;k++){ double t=now(); r=f(); best=min(best,now()-t);} printf("%-16s %7.1f ms  %lld\n",nm,best*1e3,(long long)r); };
  run("runs_hand",runs_hand); run("runs_push",runs_push_use); run("runs_pull",runs_pull_use);
  run("zip_hand",zip_hand); run("zip_pull",zip_pull); run("zip_mixed",zip_mixed);
  run("pre_rec_hand",[]{acc=0; dfs_hand(0,0); return acc;}); run("pre_rec_push",pre_push_use); run("pre_stack_hand",pre_stack_hand); run("pre_stack_pull",pre_pull_use);
}
