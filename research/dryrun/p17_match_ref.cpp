#include <bits/stdc++.h>
using namespace std;
int main(){string t,p;cin>>t>>p;int c=0;for(size_t i=0;i+p.size()<=t.size();i++)if(t.compare(i,p.size(),p)==0)c++;cout<<c<<"\n";}
