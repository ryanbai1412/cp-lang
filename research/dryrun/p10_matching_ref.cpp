#include <bits/stdc++.h>
using namespace std;
int n;vector<vector<int>>g;vector<int>vis;int ans=0;
int main(){scanf("%d",&n);g.resize(n+1);for(int i=1;i<n;i++){int a,b;scanf("%d %d",&a,&b);g[a].push_back(b);g[b].push_back(a);}
vector<int>ord,par(n+1,0);ord.push_back(1);par[1]=-1;for(size_t i=0;i<ord.size();i++){int u=ord[i];for(int v:g[u])if(v!=par[u]){par[v]=u;ord.push_back(v);}}
vector<char>m(n+1,0);for(int i=n-1;i>0;i--){int u=ord[i];if(!m[u]&&!m[par[u]]){m[u]=m[par[u]]=1;ans++;}}printf("%d\n",ans);}
