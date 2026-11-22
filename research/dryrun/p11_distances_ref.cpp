#include <bits/stdc++.h>
using namespace std;
int main(){int n;scanf("%d",&n);vector<vector<int>>g(n+1);for(int i=1;i<n;i++){int a,b;scanf("%d %d",&a,&b);g[a].push_back(b);g[b].push_back(a);}
for(int s=1;s<=n;s++){vector<int>d(n+1,-1);d[s]=0;queue<int>q;q.push(s);long long t=0;while(!q.empty()){int u=q.front();q.pop();t+=d[u];for(int v:g[u])if(d[v]<0){d[v]=d[u]+1;q.push(v);}}printf("%lld ",t);}puts("");}
