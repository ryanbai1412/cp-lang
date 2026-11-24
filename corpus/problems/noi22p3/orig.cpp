// LOJ 3849 accepted submission 2362244 (100 points, g++ C++17) after LOJ's own count.h grader, which reads
// `T n q M`, the parents and the queries, and prints each answer's edge-set hash or the limit it broke.
#include <bits/stdc++.h>

using namespace std;

struct info {
    constexpr static const unsigned seed[8] = {
        135708411,
        761037989,
        263742115,
        478706569,
        1024124597,
        483490357,
        670166970,
        571663667};
    unsigned val;
    unsigned poi[2];

    inline void encode() {
        poi[0] += val & 262143;
        poi[0] ^= val & 262143;
        poi[1] += val & 262143;
        poi[1] ^= val & 262143;
        val ^= seed[0];
        val += seed[1];
        val ^= seed[2];
        val += seed[3];
        val ^= seed[4];
        val += seed[5];
        val ^= seed[6];
        val += seed[7];
    }

    inline void decode() {
        val -= seed[7];
        val ^= seed[6];
        val -= seed[5];
        val ^= seed[4];
        val -= seed[3];
        val ^= seed[2];
        val -= seed[1];
        val ^= seed[0];
        poi[0] ^= val & 262143;
        poi[0] -= val & 262143;
        poi[1] ^= val & 262143;
        poi[1] -= val & 262143;
    }

    inline void operator=(unsigned x) {
        val = x;
        encode();
    }

    bool operator==(const info& a) const {
        return val == a.val && poi[0] == a.poi[0] && poi[1] == a.poi[1];
    }
};

const info emptyinfo = info{0, (unsigned)-1, (unsigned)-1};
info MR(info a, info b);
info MC(info a, info b);
void init(int T, int n, int q, vector<int> dad, vector<info> ve, int M);
bool isempty(info a);
info ask(int x, int d);

namespace mugifsucug {
    const unsigned mo = 998244853, N = 2e5 + 5;
    info Cost;
    unsigned mi[N];
    int val_n;

    inline void init(int n, vector<int> dad, vector<info>& ve) {
        val_n = n;
        int i;
        for (i = *mi = 1; i < N; ++i) mi[i] = 2ull * mi[i - 1] % mo;
        for (i = 2; i <= n; ++i) ve[i - 2] = info{mi[i], (unsigned)i, (unsigned)dad[i - 2]}, ve[i - 2].encode();
        Cost = 0;
    }

    inline bool equal(const info& a, const info& b) {
        return a.val == b.val;
    }
}  // namespace mugifsucug

info MR(info a, info b) {
    mugifsucug::Cost.decode();
    ++mugifsucug::Cost.val;
    if (mugifsucug::Cost.val > 6e7) {
        cout << "Use more than 6e7 MR/MC operations!!" << endl;
        exit(0);
    }
    mugifsucug::Cost.encode();
    if (mugifsucug::equal(a, emptyinfo)) return b;
    if (mugifsucug::equal(b, emptyinfo)) return a;
    a.decode();
    b.decode();
    int x = -1, y = -1;
    for (int i : {0, 1})
        for (int j : {0, 1})
            if (a.poi[i] == b.poi[j]) x = i, y = j;
    if (x == -1) {
        cout << "Illegal MR operation!!" << endl;
        exit(0);
    }
    a.val = (a.val + b.val) % mugifsucug::mo;
    a.encode();
    return a;
}

info MC(info a, info b) {
    mugifsucug::Cost.decode();
    ++mugifsucug::Cost.val;
    if (mugifsucug::Cost.val > 6e7) {
        cout << "Use more than 6e7 MR/MC operations!!" << endl;
        exit(0);
    }
    mugifsucug::Cost.encode();
    if (mugifsucug::equal(a, emptyinfo)) return b;
    if (mugifsucug::equal(b, emptyinfo)) return a;
    a.decode();
    b.decode();
    int x = -1, y = -1;
    for (int i : {0, 1})
        for (int j : {0, 1})
            if (a.poi[i] == b.poi[j]) x = i, y = j;
    if (x == -1) {
        cout << "Illegal MC operation!!" << endl;
        exit(0);
    }
    a.val = (a.val + b.val) % mugifsucug::mo;
    a.poi[x] = b.poi[y ^ 1];
    a.encode();
    return a;
}

bool isempty(info a) {
    return a.val == emptyinfo.val && a.poi[0] == emptyinfo.poi[0] && a.poi[1] == emptyinfo.poi[1];
}

int main() {
    int T, n, q, M, i, x, d;
    ios::sync_with_stdio(0);
    cin.tie(0);
    cin >> T >> n >> q >> M;
    vector<int> dad(n - 1);
    vector<info> ve(n - 1);
    for (i = 0; i < n - 1; ++i) cin >> dad[i];
    mugifsucug::init(n, dad, ve);
    // cerr<<233<<endl;
    init(T, n, q, dad, ve, M);
    // cerr<<233<<endl;
    mugifsucug::Cost.decode();
    int mx = 0, ov = mugifsucug::Cost.val;
    mugifsucug::Cost.encode();
    if (ov > 3e7) {  // gai can shu
        cout << "too many operations in INIT" << endl;
        exit(0);
    }
    for (; q--;) {
        cin >> x >> d;
        mugifsucug::Cost.decode();
        int ovv = mugifsucug::Cost.val;
        mugifsucug::Cost.encode();
        auto ass = ask(x, d);
        ass.decode();
        cout << ass.val << '\n';
        mugifsucug::Cost.decode();
        mx = max(mx, (int)mugifsucug::Cost.val - ovv);
        mugifsucug::Cost.encode();
        if (mx > M) {
            cout << "too many operations in ASK" << endl;
            exit(0);
        }
    }
}

#include<bits/stdc++.h>
using namespace std;
typedef pair<int,int> pii;
typedef vector<pii>::iterator iter;
constexpr int N=200000;
inline info rake(info x,info y){return isempty(x)?y:isempty(y)?x:MR(x,y);}
inline info compress(info x,info y){return isempty(x)?y:isempty(y)?x:MC(x,y);}

int siz[N+1];
vector<int> E[N+1];
void dfs(int u){
    siz[u]=1; for(int v :E[u])dfs(v),siz[u]+=siz[v];
    if(siz[u]>1)swap(E[u][0],*max_element(E[u].begin(),E[u].end(),[&](int x,int y){return siz[x]<siz[y];}));
}

int cnode,r,t[N*2],lc[N*2],rc[N*2],fa[N*2]; // t: 0 = rake, 1 = compress
vector<info> f[N*2][2],g[N*2][2],tg[N*2];
inline info get(vector<info> &v,int d){return d<0?emptyinfo:d>=(int)v.size()?v.back():v[d];}
int len[N*2],dis[N*2],lenu[N*2],lenv[N*2];
int merge(iter L,iter R,int t){
    if(next(L)==R)return L->first; iter mid=next(L);
    int sum=accumulate(mid,R,-L->second,[&](int x,pii y){return x+y.second;}),u=++cnode;
    while(next(mid)!=R&&sum>mid->second)sum-=mid->second*2,mid++;
    return ::t[u]=t,fa[lc[u]=merge(L,mid,t)]=fa[rc[u]=merge(mid,R,t)]=u;
}
int build(int u){
    vector<pii> v1; v1.emplace_back(u,1);
    while(!E[u].empty()){
        vector<pii> v2; v2.emplace_back(E[u][0],1);
        for(int v :E[u])if(v!=E[u][0])v2.emplace_back(build(v),siz[v]);
        v1.emplace_back(merge(v2.begin(),v2.end(),0),siz[u]-siz[E[u][0]]),u=E[u][0];
    }
    return merge(v1.begin(),v1.end(),1);
}

void initInfo(int u){
    if(!lc[u])return len[u]=dis[u]=lenu[u]=lenv[u]=1,void();
    initInfo(lc[u]),initInfo(rc[u]);
    if(int x=lc[u],y=rc[u];t[u]){
        dis[u]=dis[x]+dis[y],len[u]=max({len[x],len[y],lenv[x]+lenu[y]});
        lenu[u]=max(lenu[x],dis[x]+lenu[y]),lenv[u]=max(lenv[y],len[y]+lenv[x]);
    }
    else{
        dis[u]=dis[x],len[u]=max({len[x],len[y],lenu[x]+lenu[y]});
        lenu[u]=max(lenu[x],lenu[y]),lenv[u]=max(lenv[x],len[x]+lenu[y]);
    }
    f[u][0].assign(len[u]+1,emptyinfo),f[u][1].assign(len[u]+1,emptyinfo);
}

void calcF(int u,vector<info> &w){
    if(!lc[u]){
        f[u][0].assign(2,emptyinfo),f[u][1].assign(2,emptyinfo);
        return f[u][0][1]=f[u][1][1]=u==1?emptyinfo:w[u-2],void();
    }
    calcF(lc[u],w),calcF(rc[u],w);
    if(int x=lc[u],y=rc[u];t[u])for(int i=0;i<=len[u];i++){
        f[u][0][i]=compress(get(f[x][0],i),get(f[y][0],i-dis[x]));
        f[u][1][i]=compress(get(f[y][1],i),get(f[x][1],i-dis[y]));
    }
    else for(int i=0;i<=len[u];i++){
        f[u][0][i]=rake(get(f[x][0],i),get(f[y][0],i));
        f[u][1][i]=rake(get(f[x][1],i),get(f[y][0],i-dis[x]));
    }
}
void calcG(int u){
    if(!lc[u])return; int x=lc[u],y=rc[u];
    for(int i :{x,y})for(int j :{0,1})g[i][j].assign(len[u]+1,emptyinfo);
    if(t[u])for(int i=0;i<=len[u];i++){
        g[x][0][i]=get(g[u][0],i),g[x][1][i]=compress(get(f[y][0],i),get(g[u][1],i-dis[y]));
        g[y][1][i]=get(g[u][1],i),g[y][0][i]=compress(get(f[x][1],i),get(g[u][0],i-dis[x]));
    }
    else for(int i=0;i<=len[u];i++){
        g[x][0][i]=rake(get(g[u][0],i),get(f[y][0],i)),g[x][1][i]=get(g[u][1],i);
        g[y][0][i]=rake(get(g[u][0],i),compress(get(f[x][0],i),get(g[u][1],i-dis[x]))),g[y][1][i]=emptyinfo;
    }
    calcG(lc[u]),calcG(rc[u]);
}

void init(int tid,int n,int q,vector<int> p,vector<info> w,int lim){
    cnode=n;
    for(int i=2;i<=n;i++)E[p[i-2]].push_back(i);
    dfs(1),initInfo(r=build(1)),calcF(r,w);
    g[r][0].assign(1,emptyinfo),g[r][1].assign(1,emptyinfo),calcG(r);
    for(int u=1;u<=cnode;u++){tg[u]=g[u][1]; for(info &i :tg[u])i=compress(i,f[u][0].back());}
}
info ask(int u,int d){
    int dx=1,dy=0;
    while(fa[u]&&len[fa[u]]<=d){
        if(int v=fa[u];t[v])u==lc[v]?(dy+=dis[rc[v]]):(dx+=dis[lc[v]]);
        else if(u==rc[v])dy=dx+dis[lc[v]];
        u=fa[u];
    }
    return compress(get(g[u][0],d-dx),get(tg[u],d-dy));
}
