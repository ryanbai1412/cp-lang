// Exhaustive backtracking over palindromes below 1e6, pruned by the best sum so far: any
// optimal set sums to at most 98989, so every member is below 1e6 and the search is complete.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
vector<ll> pal; ll best=LLONG_MAX; vector<ll> cur,ans;
bool isp(ll x){char b[24];int n=sprintf(b,"%lld",x);for(int i=0;i<n/2;i++)if(b[i]!=b[n-1-i])return false;return true;}
void dfs(int from, vector<ll>& sums, ll sum){
    int need=9-cur.size();
    if(!need){ if(sum<best){best=sum;ans=cur;} return;}
    for(int i=from;i<(int)pal.size();i++){
        ll x=pal[i];
        if(sum+x*need+(ll)need*(need-1)/2>=best) break;
        bool ok=true; for(ll s:sums) if(!isp(s+x)){ok=false;break;}
        if(!ok) continue;
        vector<ll> ns=sums; for(ll s:sums) ns.push_back(s+x);
        cur.push_back(x); dfs(i+1,ns,sum+x); cur.pop_back();
    }
}
int main(){
    for(ll x=1;x<1000000;x++) if(isp(x)) pal.push_back(x);
    vector<ll> s={0}; dfs(0,s,0);
    puts("0"); for(ll x:ans) printf("%lld\n",x);
}
