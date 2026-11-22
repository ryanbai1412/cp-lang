#include <bits/stdc++.h>
using namespace std;
int p[1001][1001];
int main(){int n,q;cin>>n>>q;for(int i=1;i<=n;i++){string s;cin>>s;for(int j=1;j<=n;j++)p[i][j]=p[i-1][j]+p[i][j-1]-p[i-1][j-1]+(s[j-1]=='*');}
while(q--){int a,b,c,d;cin>>a>>b>>c>>d;cout<<p[c][d]-p[a-1][d]-p[c][b-1]+p[a-1][b-1]<<"\n";}}
