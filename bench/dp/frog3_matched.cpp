#include <bits/stdc++.h>
using namespace std; typedef long long ll;
struct L{ll k,m; ll at(ll x)const{return k*x+m;}};
struct C{L l; bool has; int lo,hi;};
vector<C> t; ll LO,HI;
int kid(int i,bool r){int k=r?t[i].hi:t[i].lo; if(k>=0)return k; t.push_back({{0,0},false,-1,-1}); k=t.size()-1; if(r)t[i].hi=k; else t[i].lo=k; return k;}
void add(L f){int i=0; ll l=LO,r=HI; while(true){ if(!t[i].has){t[i].l=f;t[i].has=true;return;} L o=t[i].l; ll mid=l+(r-l)/2; bool lf=f.at(l)<o.at(l), md=f.at(mid)<o.at(mid); if(md){t[i].l=f; f=o;} if(l==r)return; if(lf!=md){i=kid(i,false); r=mid;} else {i=kid(i,true); l=mid+1;}}}
bool query(ll x, ll&out){int i=0; ll l=LO,r=HI; bool any=false; while(i>=0){ if(t[i].has){ll v=t[i].l.at(x); if(!any||v<out){out=v;any=true;}} ll mid=l+(r-l)/2; if(x<=mid){i=t[i].lo; r=mid;} else {i=t[i].hi; l=mid+1;}} return any;}
int main(){int n; ll c; scanf("%d %lld",&n,&c); LO=1;HI=1000000; t.push_back({{0,0},false,-1,-1}); ll cost=0; for(int i=0;i<n;i++){ll x; scanf("%lld",&x); ll b; if(query(x,b)) cost=b+x*x+c; add({-2*x,cost+x*x});} printf("%lld\n",cost);}
