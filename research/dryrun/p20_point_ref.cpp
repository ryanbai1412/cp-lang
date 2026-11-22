#include <bits/stdc++.h>
using namespace std;
int main(){int t;cin>>t;while(t--){long long a,b,c,d,e,f;cin>>a>>b>>c>>d>>e>>f;__int128 x=(__int128)(c-a)*(f-b)-(__int128)(d-b)*(e-a);puts(x>0?"LEFT":x<0?"RIGHT":"TOUCH");}}
