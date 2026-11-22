#include <bits/stdc++.h>
using namespace std;
int main(){int n,m;scanf("%d %d",&n,&m);vector<vector<pair<int,long long>>>g(n+1);for(int i=0;i<m;i++){int a,b;long long c;scanf("%d %d %lld",&a,&b,&c);g[a].push_back({b,c});}
vector<long long>d(n+1,LLONG_MAX);priority_queue<pair<long long,int>,vector<pair<long long,int>>,greater<>>pq;d[1]=0;pq.push({0,1});
while(!pq.empty()){auto[du,u]=pq.top();pq.pop();if(du!=d[u])continue;for(auto[v,w]:g[u])if(du+w<d[v]){d[v]=du+w;pq.push({d[v],v});}}
for(int i=1;i<=n;i++)printf("%lld ",d[i]);puts("");}
