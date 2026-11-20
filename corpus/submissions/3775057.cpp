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

struct Comp {
	int ind = 0;
	map<int, int> m {};

	int operator()(int x) {
		int &ans = m[x];
		if(!ans) {
			ans = ++ind;
		}
		return ans - 1;
	}
};

const int maxn = 1e6;

bool vis[maxn];
vector<int> order;
vector<pair<int, int>> graph[maxn];

void dfs(int u, int t) {
	vis[u] = true;
	for(auto &[v, ct]: graph[u]) {
		if(!vis[v] && t == ct) {
			dfs(v, t);
		}
	}
	order.push_back(u);
}

struct mask {
	int x;

	bool operator[](int ind) {
		return !!((x >> ind) & 1);
	}
};

void slow(int q) {
	map<int, int> ccomp;
	vector<int> vals;
	vector<pair<int, int>> eq, neq;
	while(q--) {
		int u, v, t;
		cin >> u >> v >> t;
		vals.push_back(u);
		vals.push_back(v);
		if(t) {
			eq.emplace_back(u, v);
		}else {
			neq.emplace_back(u, v);
		}
	}
	sort(begin(vals), end(vals));
	vals.erase(unique(begin(vals), end(vals)), vals.end());
	int n = sz(vals);
	for(int i = 0; i < n; i++) {
		ccomp[vals[i]] = i;
	}
	for(auto &[u, v]: eq) {
		u = ccomp[u];
		v = ccomp[v];
	}
	for(int ii = 0; ii < (1 << n); ii++) {
		mask i {ii};
		for(auto &[u, v]: eq) {
			if(i[u] != i[v]) {
				goto loop;
			}
		}
		for(auto &[u, v]: neq) {
			if(i[u] == i[v]) {
				goto loop;
			}
		}
		cout << "YES" << endl;
		return;
		loop:;
	}
	cout << "NO" << endl;
}

void solve() {
	int q;
	cin >> q;
	if(q <= 10) {
		slow(q);
		return;
	}
	Comp comp;
	auto push = [&](int u, int v) -> void {
		dbg("EDGE", u, v);
		graph[u].emplace_back(v, 0);
		graph[v].emplace_back(u, 1);
	};
	auto implies = [&](int u, int v) -> void {
		push(u, v);
		push(v ^ 1, u ^ 1);
	};
	for(int i = 0; i < q; i++) {
		int u, v, t;
		cin >> u >> v >> t;
		u = comp(u);
		v = comp(v);
		if(t) {
			implies(u * 2, v * 2);
			implies(v * 2, u * 2);
		}else {
			implies(u * 2, v * 2 + 1);
			implies(u * 2 + 1, v * 2);
		}
	}
	int n = comp.ind * 2;
	auto reset = [&]() -> void {
		for(int i = 0; i < n; i++) {
			graph[i].clear();
		}
	};
	memset(vis, 0, sizeof(vis));
	order.clear();
	for(int i = 0; i < n; i++) {
		if(!vis[i]) {
			dfs(i, 0);
		}
	}
	vector<int> tsort(order.rbegin(), order.rend());
	int cind = 0;
	memset(vis, 0, sizeof(vis));
	int comps[n];
	memset(comps, -1, sizeof(comps));
	for(auto &a: tsort) {
		if(!vis[a]) {
			order.clear();
			dfs(a, 1);
			int cxind = cind++;
			for(auto &b: order) {
				comps[b] = cxind;
			}
		}
	}
	for(int i = 0; i < n; i += 2) {
		assert(comps[i] != -1 && comps[i + 1] != -1);
		if(comps[i] == comps[i + 1]) {
			cout << "NO" << endl;
			reset();
			return;
		}
	}
	cout << "YES" << endl;
	reset();
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}