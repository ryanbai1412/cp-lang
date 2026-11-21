#include <bits/stdc++.h>
using namespace std; typedef long long ll;
int main(){int n; scanf("%d",&n); vector<ll> pre(n+1,0); for(int i=0;i<n;i++){ll x; scanf("%lld",&x); pre[i+1]=pre[i]+x;}
 vector<vector<ll>> dp(n+1,vector<ll>(n+1,0)); vector<vector<int>> opt(n+1,vector<int>(n+1,0));
 for(int i=0;i<n;i++) opt[i][i+1]=i;
 for(int len=2;len<=n;len++) for(int i=0;i+len<=n;i++){int j=i+len; int from=max(opt[i][j-1],i+1); int to=max(from,min(opt[i+1][j],j-1)); ll best=dp[i][from]+dp[from][j]; int at=from; for(int k=from+1;k<=to;k++){ll v=dp[i][k]+dp[k][j]; if(v<best){best=v;at=k;}} dp[i][j]=best+pre[j]-pre[i]; opt[i][j]=at;}
 printf("%lld\n",dp[0][n]);}
