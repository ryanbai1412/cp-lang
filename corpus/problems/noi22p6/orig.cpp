// LOJ 3852 accepted submission 2362881 (100 points, g++ C++17) without its qip.in/qip.out freopen, the reference the answers come from.
#include<bits/stdc++.h>
#define pb emplace_back
#define fi first
#define se second
#define mp make_pair
using namespace std;
typedef long long ll;
typedef pair<int,int> pii; 
void cmax(int &x,int y){ x=x>y?x:y; }
void cmin(int &x,int y){ x=x<y?x:y; }
const int maxn=610;
const int maxm=1e6+10;
const ll Vg=1e6;
const ll inf=1e13;
int L[maxn],R[maxn],n,K,m,q; ll val[maxn];
int cnt[6],U[maxm],V[maxm],W[maxm];
vector<pii> vec; int id[maxn][2],c=0;
struct DSU{
	int fa[maxn],sz[maxn];
	void init(){ for(int i=1;i<=n;i++) fa[i]=i,sz[i]=1; }
	int find(int u){ return fa[u]==u?u:fa[u]=find(fa[u]); }
	bool merge(int u,int v){
		u=find(u); v=find(v);
		if(u==v) return 0;
		if(sz[u]<sz[v]) swap(u,v);
		fa[v]=u; sz[u]+=sz[v]; return 1;
	}
}dsu;
struct Dinic{
	struct edge{
		int to,Next; ll f;
	}edges[maxm]; queue<int> q;
	int head[maxm],now[maxm],d[maxm],tot;
	void init(){ tot=1;	for(int i=1;i<=c+2;i++) head[i]=0; }//别忘记调用了 
	void add(int u,int v,ll f){
		edges[++tot]=(edge){v,head[u],f}; head[u]=tot;
		edges[++tot]=(edge){u,head[v],0}; head[v]=tot;
	}
	bool bfs(int s,int t){
		while(q.size()) q.pop(); for(int i=1;i<=c+2;i++) d[i]=0; 
		d[s]=1; q.push(s); now[s]=head[s];//清空 now(s) 
		while(!q.empty()){
			int u=q.front(); q.pop();
			for(int i=head[u];i;i=edges[i].Next){
				int v=edges[i].to; if(d[v]||!edges[i].f) continue;//注意两个条件 
				now[v]=head[v]; d[v]=d[u]+1; q.push(v);//清空 now(v) 
				if(v==t) return 1; 
			}
		}
		return 0;
	}
	ll dinic(int s,int t,ll flow){
		if(s==t) return flow;
		ll rest=flow,k;
		for(int i=now[s];i&&rest;i=edges[i].Next){//判断 rest 是否有剩余 
			now[s]=i; int v=edges[i].to;//动态更新 now 
			if(!edges[i].f||d[v]!=d[s]+1) continue;// 注意 d 的条件 
			k=dinic(v,t,min(rest,edges[i].f));
			if(!k) d[v]=0; //注意 
			rest-=k; edges[i].f-=k; edges[i^1].f+=k;
		}
		return flow-rest;
	}
	pii solve(int s,int t){
		ll maxflow=0,flow;
		while(bfs(s,t))
			while(flow=dinic(s,t,inf)) maxflow+=flow;
	//	cout<<maxflow<<endl;
		int c2=0,c4=0; bfs(s,t);
		for(int i=1;i<=n;i++){
			if(dsu.find(i)!=i||L[i]==1||R[i]==5) continue;
			if(!d[id[i][0]]) c2+=dsu.sz[i];
			else if(d[id[i][1]]) c4+=dsu.sz[i];
		}
		return mp(c2,c4);
	}
}DC;
void init(){
	dsu.init(); vec.clear(); c=0;
	for(int i=1;i<=5;i++) cnt[i]=0;
}
namespace K3{
	void solve(){
		ll ans; 
		while(q--){
			cin>>val[2];
			int c2=n-cnt[1]-cnt[3];
			ans=Vg*(1ll*n*n-2ll*cnt[1]*cnt[3])+c2*val[2];
			cout<<ans<<"\n";
		}
	}
}
namespace K4{
	void solve(){
		ll ans; 
		while(q--){
			cin>>val[2]>>val[3];
			int r=n-cnt[1]-cnt[2]-cnt[3]-cnt[4];
			int c2=cnt[2]+r;
			ans=Vg*(1ll*n*n-2ll*cnt[1]*(cnt[3]+cnt[4])-2ll*c2*cnt[4])+c2*val[2]+cnt[3]*val[3];
			int c3=cnt[3]+r;
			ans=max(ans,Vg*(1ll*n*n-2ll*cnt[1]*(c3+cnt[4])-2ll*cnt[2]*cnt[4])+cnt[2]*val[2]+c3*val[3]);
			cout<<ans<<"\n";
		}
	}
}
bool chk(pii A,pii B,pii C){
	pii D=mp(B.fi-A.fi,B.se-A.se);
	pii E=mp(C.fi-A.fi,C.se-A.se);
	if(1ll*D.fi*E.se-1ll*D.se*E.fi>0) return 1;
	else return 0;
}
void solve(pii P,pii Q){
	DC.init(); int S=c+1,T=c+2;
//	cout<<P.fi<<" "<<P.se<<" "<<Q.fi<<" "<<Q.se<<endl;
	int X=Q.se-P.se+n,Y=P.fi-Q.fi+n;
//	cout<<X<<" "<<n<<" "<<Y<<endl;
	for(int i=1;i<=n;i++){
		if(dsu.find(i)!=i||R[i]==1||L[i]==5) continue;
		int sz=dsu.sz[i]; //cout<<L[i]<<" "<<R[i]<<endl;
		DC.add(S,id[i][0],(L[i]==2?sz*X:inf));
		DC.add(id[i][0],id[i][1],(L[i]<=3&&R[i]>=3)?sz*n:inf);
		DC.add(id[i][1],T,(R[i]==4?sz*Y:inf));
	}
	for(int i=1;i<=m;i++){
		int u=dsu.find(U[i]),v=dsu.find(V[i]);
		if(W[i]!=1||u==v) continue;
		DC.add(id[u][1],id[v][0],inf); 
		DC.add(id[v][1],id[u][0],inf);
	}
	pii R=DC.solve(S,T);
//	cout<<R.fi<<" "<<R.se<<endl;
	if(!chk(P,Q,R)) return ;
	vec.pb(R); solve(P,R); solve(R,Q); 
}
ll calc(){
	ll res=0;
	for(int i=2;i<=K-1;i++) res+=cnt[i]*val[i];
	for(int i=1;i<=K;i++){
		res+=Vg*cnt[i]*cnt[i];
		if(i+1<=K) res+=Vg*cnt[i]*cnt[i+1];
		if(i) res+=Vg*cnt[i-1]*cnt[i];
	}
	return res;
}
void solve(){
	cin>>K>>n>>m>>q; init();
//	cout<<K<<" "<<n<<" "<<m<<" "<<q<<endl;
	for(int i=1;i<=n;i++)
		for(auto j:{0,1})
			id[i][j]=++c;
	for(int i=1;i<=n;i++) cin>>L[i]>>R[i];
	for(int i=1;i<=m;i++) cin>>U[i]>>V[i]>>W[i];
	for(int i=1;i<=n;i++){
		for(int j=1;j<=m;j++){
			int x=U[j],y=V[j],w=W[j];
			cmax(L[x],L[y]-w); cmin(R[x],R[y]+w);
			cmax(L[y],L[x]-w); cmin(R[y],R[x]+w);
		}
	}
	int mx2=0,mx4=0;
	for(int i=1;i<=n;i++){
		if(L[i]<K&&R[i]>1) cmax(L[i],2),cmin(R[i],K-1);
		if(L[i]==R[i]) cnt[L[i]]++;
		mx2+=(L[i]==2); mx4+=(R[i]==4);
	}
	if(K==3){ K3::solve(); return ; } 
	if(K==4){ K4::solve(); return ; }
	for(int i=1;i<=m;i++)
		if(!W[i]) dsu.merge(U[i],V[i]); 
	vec.pb(mp(cnt[2],cnt[4])); vec.pb(mp(mx2,cnt[4])); vec.pb(mp(cnt[2],mx4));
	solve(vec[2],vec[1]);
	while(q--){
		for(auto i:{2,3,4}) cin>>val[i];
		ll ans=0;
		for(auto z:vec){
			cnt[2]=z.fi,cnt[4]=z.se;
			cnt[3]=n-cnt[1]-cnt[2]-cnt[4]-cnt[5];
			ans=max(ans,calc());
		}
		cout<<ans<<"\n";
	}
}
int main(){
	ios::sync_with_stdio(false); cin.tie(0); cout.tie(0);
	int c,T; cin>>c>>T;
	while(T--) solve();
	return 0;
}