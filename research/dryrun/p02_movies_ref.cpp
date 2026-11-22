#include <bits/stdc++.h>
using namespace std;
int main(){int n;cin>>n;vector<pair<int,int>>v(n);for(auto&[a,b]:v)cin>>a>>b;sort(v.begin(),v.end(),[](auto x,auto y){return x.second<y.second;});int e=0,c=0;for(auto[a,b]:v)if(a>=e){c++;e=b;}cout<<c<<"\n";}
