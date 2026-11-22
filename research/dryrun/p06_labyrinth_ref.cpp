#include <bits/stdc++.h>
using namespace std;
int main(){int n,m;cin>>n>>m;vector<string>g(n);for(auto&s:g)cin>>s;int sr,sc,tr,tc;
for(int i=0;i<n;i++)for(int j=0;j<m;j++){if(g[i][j]=='A')sr=i,sc=j;if(g[i][j]=='B')tr=i,tc=j;}
vector<vector<int>>d(n,vector<int>(m,-1));queue<pair<int,int>>q;q.push({sr,sc});d[sr][sc]=0;
int dr[]={-1,0,1,0},dc[]={0,1,0,-1};while(!q.empty()){auto[r,c]=q.front();q.pop();for(int k=0;k<4;k++){int a=r+dr[k],b=c+dc[k];if(a<0||b<0||a>=n||b>=m||g[a][b]=='#'||d[a][b]>=0)continue;d[a][b]=d[r][c]+1;q.push({a,b});}}
if(d[tr][tc]<0){puts("NO");return 0;}printf("YES\n%d\n",d[tr][tc]);}
