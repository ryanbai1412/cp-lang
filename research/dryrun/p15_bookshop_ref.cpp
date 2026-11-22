#include <bits/stdc++.h>
using namespace std;
int main(){int n,x;cin>>n>>x;vector<int>h(n),s(n);for(auto&v:h)cin>>v;for(auto&v:s)cin>>v;int best=0;
for(int m=0;m<(1<<n);m++){int c=0,p=0;for(int i=0;i<n;i++)if(m>>i&1){c+=h[i];p+=s[i];}if(c<=x)best=max(best,p);}cout<<best<<"\n";}
