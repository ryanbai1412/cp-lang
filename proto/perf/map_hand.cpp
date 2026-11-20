#include <bits/stdc++.h>
using namespace std; using ll = long long;
int main(){ int n; cin>>n; vector<ll> last(n,-1); ll x=12345; int c=0;
for(int i=0;i<n;i++){ x=(x*1103515245+12345)%2147483648LL; int k=x%n; if(last[k]<0) c++; last[k]=i; }
ll s=0; for(int k=0;k<n;k++) if(last[k]>=0) s+=k^last[k]; cout<<c<<" "<<s<<"\n"; }
