#include <bits/stdc++.h>
int main(){int t;scanf("%d",&t);while(t--){long long n;scanf("%lld",&n);int c=0;for(int d=1;d<=9;d++)for(long long x=d;x<=n;x=x*10+d)c++;printf("%d\n",c);}}
