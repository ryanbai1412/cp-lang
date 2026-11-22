#include <bits/stdc++.h>
using namespace std;
int main(){int n,q;scanf("%d %d",&n,&q);vector<long long>a(n+1);for(int i=1;i<=n;i++)scanf("%lld",&a[i]);
while(q--){int k,l,r;long long x;scanf("%d %d %d",&k,&l,&r);if(k==3){long long s=0;for(int i=l;i<=r;i++)s+=a[i];printf("%lld\n",s);continue;}scanf("%lld",&x);for(int i=l;i<=r;i++)a[i]=k==1?a[i]+x:x;}}
