#include <bits/stdc++.h>
using namespace std;
int main(){int n,q;scanf("%d %d",&n,&q);vector<long long>a(n+1);for(int i=1;i<=n;i++)scanf("%lld",&a[i]);
while(q--){int k;long long x,y;scanf("%d %lld %lld",&k,&x,&y);if(k==1)a[x]=y;else{long long s=0;for(int i=x;i<=y;i++)s+=a[i];printf("%lld\n",s);}}}
