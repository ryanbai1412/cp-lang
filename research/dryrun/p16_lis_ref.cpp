#include <bits/stdc++.h>
using namespace std;
int main(){int n;cin>>n;vector<long long>a(n);for(auto&v:a)cin>>v;vector<int>d(n,1);int b=0;
for(int i=0;i<n;i++){for(int j=0;j<i;j++)if(a[j]<a[i])d[i]=max(d[i],d[j]+1);b=max(b,d[i]);}cout<<b<<"\n";}
