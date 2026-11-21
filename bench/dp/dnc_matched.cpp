#include <bits/stdc++.h>
using namespace std; typedef long long ll;
int n,k; vector<ll> pre;
struct R{bool has; ll val; int at;};
vector<optional<ll>> layer(const vector<optional<ll>>&prev){
  vector<R> out(n+1,{false,0,0});
  vector<array<int,4>> todo={{0,n+1,0,n+1}};
  while(!todo.empty()){auto [lo,hi,from,to]=todo.back(); todo.pop_back(); if(lo>=hi)continue; int mid=(lo+hi)/2; R best={false,0,0};
    for(int j=from;j<to;j++){ if(j<mid&&prev[j]){ ll v=*prev[j]+(pre[mid]-pre[j])*(pre[mid]-pre[j]); if(!best.has||v<best.val)best={true,v,j};}}
    out[mid]=best; int cut=best.has?best.at:from; todo.push_back({lo,mid,from,best.has?cut+1:to}); todo.push_back({mid+1,hi,cut,to});}
  vector<optional<ll>> r(n+1); for(int i=0;i<=n;i++) if(out[i].has) r[i]=out[i].val; return r;}
int main(){scanf("%d %d",&n,&k); pre.assign(n+1,0); for(int i=0;i<n;i++){ll x; scanf("%lld",&x); pre[i+1]=pre[i]+x;} vector<optional<ll>> L(n+1); L[0]=0; for(int t=0;t<k;t++) L=layer(L); printf("%lld\n",*L[n]);}
