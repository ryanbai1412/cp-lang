#include <bits/stdc++.h>
using namespace std;
int main(){long long n,x;cin>>n>>x;vector<long long>a(n);for(auto&v:a)cin>>v;long long c=0;
for(int i=0;i<n;i++){long long s=0;for(int j=i;j<n;j++){s+=a[j];if(s==x)c++;if(s>x)break;}}cout<<c<<"\n";}
