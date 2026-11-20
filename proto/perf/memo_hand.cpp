#include <bits/stdc++.h>
using namespace std; using ll = long long;
int n; vector<ll> m;
ll f(int i,int j){ ll& r=m[(size_t)i*n+j]; if(r>=0) return r; if(i>0&&j>0) return r=(f(i-1,j)+f(i,j-1))%1000000007; return r=1; }
int main(){ cin>>n; m.assign((size_t)n*n,-1); cout<<f(n-1,n-1)<<"\n"; }
