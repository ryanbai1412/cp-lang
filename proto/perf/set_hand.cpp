#include <bits/stdc++.h>
using namespace std; using ll = long long;
int main(){ int n; cin>>n; vector<char> seen(1000000); ll x=12345; int c=0;
for(int i=0;i<n;i++){ x=(x*1103515245+12345)%2147483648LL; int v=x%1000000; if(!seen[v]) seen[v]=1,c++; }
cout<<c<<"\n"; }
