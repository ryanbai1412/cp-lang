#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 100;

long dist[maxn];
vector<pair<int, int>> graph[maxn];

struct SPG {
	vector<int> ggraph[maxn];

	SPG(int source) {
		memset(dist, 0x3f, sizeof(dist));
		mpq<pair<long, int>> pq;
		auto push = [&](int u, long d) -> void {
			if(d < dist[u]) {
				dist[u] = d;
				pq.emplace(d, u);
			}
		};
		push(source, 0);
		while(sz(pq)) {
			auto[d, u] = pq.top();
			pq.pop();
			for(auto &[v, w]: graph[u]) {
				push(v, d + w);
			}
		}
		for(int i = 0; i < maxn; i++) {
			for(auto &[u, w]: graph[i]) {
				if(dist[i] + w == dist[u]) {
					ggraph[i].push_back(u);
				}
			}
		}
	}
};

struct PCNT {
	int u;
	const SPG &g;
	long ans[maxn] {};

	PCNT(const SPG &g, int u): u(u), g(g) {
		ans[u] = 1;
		int icnt[maxn] {};
		for(int i = 0; i < maxn; i++) {
			for(auto &v: g.ggraph[i]) {
				icnt[v]++;
			}
		}
		queue<int> q;
		for(int i = 0; i < maxn; i++) {
			if(!icnt[i]) {
				q.push(i);
			}
		}
		while(sz(q)) {
			int u = q.front();
			q.pop();
			for(auto &v: g.ggraph[u]) {
				ans[v] += ans[u];
				if(!(--icnt[v])) {
					q.push(v);
				}
			}
		}
	}
};

void solve() {
	int n, m;
	cin >> n >> m;
	for(int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		u--;
		v--;
		graph[u].emplace_back(v, w);
		graph[v].emplace_back(u, w);
	}
	double ans[n] {};
	for(int s = 0; s < n; s++) {
		SPG cur(s);
		vector<PCNT> cnts;
		for(int i = 0; i < n; i++) {
			cnts.emplace_back(cur, i);
		}
		for(int t = 0; t < n; t++) {
			for(int v = 0; v < n; v++) {
				if(s != v && t != v) {
					ans[v] += double(cnts[s].ans[v] * cnts[v].ans[t]) / cnts[s].ans[t];
				}
			}
		}
	}
	for(auto &a: ans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	cout << fixed << setprecision(3);
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}