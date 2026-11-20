#include <bits/stdc++.h>
using namespace std; using ll = long long;
int main(){ int n; cin>>n; vector<ll> best(n,-1); ll x=12345;
for(int i=0;i<n;i++){ x=(x*1103515245+12345)%2147483648LL; if(x%3!=0) best[i]=x; }
ll s=0; for(int i=0;i<n;i++) s+= best[i]<0?0:best[i]; cout<<s<<"\n"; }
