#include <bits/stdc++.h>
using namespace std; typedef long long ll;
int main(){int n,k; scanf("%d %d",&n,&k); vector<pair<int,ll>> q; size_t head=0; ll total=0;
 for(int i=0;i<n;i++){ll x; scanf("%lld",&x); while(q.size()>head && !(q.back().second<x)) q.pop_back(); q.push_back({i,x}); while(q.size()>head && q[head].first<i-k+1) head++; if(head*2>q.size()&&head>64){q.erase(q.begin(),q.begin()+head);head=0;} total+=q[head].second;}
 printf("%lld\n",total);}
