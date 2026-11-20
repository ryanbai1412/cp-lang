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

struct Compressor {
	int ind = 0;
	map<int, int> m;

	int operator()(int x) {
		int &ans = m[x];
		if(!ans) {
			ans = ++ind;
		}
		return ans - 1;
	}
};

struct Tree {
	int x, y, i;
};

struct TreeInfo {
	int cy, hind, hsz;
};

struct Edge {
	int v, find, tind;
};

const int maxn = 1e6;

bool vis[maxn];
int memo[maxn];
Tree arr[maxn];
Edge nxt[maxn];
Compressor compr;
TreeInfo info[maxn];
vector<int> hlines[maxn], ngraph[maxn], igraph[maxn], order;
vector<Edge> graph[maxn];

void addEdge(int u, int v) {
	dbg("EDGE", arr[u].i, arr[v].i);
	int uy = arr[u].y, vy = arr[v].y;
	if(uy != vy) {
		if(uy > vy) {
			swap(uy, vy);
			swap(u, v);
		}
		graph[compr(uy)].push_back({arr[v].i, info[arr[u].i].hind, info[arr[v].i].hind});
	}
}

void addEdge(int i) {
	addEdge(i, i + 1);
}

int dp(int u) {
	int &ans = memo[u];
	if(ans != -1) {
		return ans;
	}
	auto &[cy, hind, hsz] = info[u];
	nxt[u] = {-1, -1, -1};
	ans = hsz;
	for(auto &[v, find, tind]: graph[cy]) {
		int cadd = dp(v);
		if(hind == find) {
			cadd += 1;
		}else if(hind < find) {
			cadd += find + 1;
		}else {
			cadd += hsz - find;
		}
		if(cadd > ans) {
			ans = cadd;
			nxt[u] = {v, find, tind};
		}
	}
	return ans;
}

void push(int u, int v) {
	static set<pair<int, int>> added;
	int csz = sz(added);
	added.insert(minmax(u, v));
	if(sz(added) == csz) {
		return;
	}
	dbg("AEDGE", u, v);
	ngraph[u].push_back(v);
	igraph[v].push_back(u);
}

void gdp(int u) {
	if(vis[u]) {
		return;
	}
	vis[u] = true;
	int ctarg = dp(u);
	auto &[cy, hind, hsz] = info[u];
	for(auto &[v, find, tind]: graph[cy]) {
		int cadd = dp(v);
		if(hind == find) {
			cadd += 1;
		}else if(hind < find) {
			cadd += find + 1;
		}else {
			cadd += hsz - find;
		}
		if(cadd == ctarg) {
			push(hlines[cy][find], hlines[info[v].cy][tind]);
			gdp(v);
		}
	}
}

void dfs(int u) {
	if(vis[u]) {
		return;
	}
	vis[u] = true;
	for(auto &v: ngraph[u]) {
		dfs(v);
	}
	order.push_back(u);
}

void solve() {
	int n;
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i].x >> arr[i].y;
		assert(arr[i].y);
		arr[i].i = i;
	}
	arr[n] = {0, 0, n};
	n++;

	for(int i = 0; i < n; i++) {
		hlines[compr(arr[i].y)].push_back(arr[i].i);
		info[i] = {compr(arr[i].y), 0, 0};
	}
	for(auto &v: hlines) {
		sort(begin(v), end(v), [&](int a, int b) -> bool {
			return arr[a].x < arr[b].x;
		});
		for(int i = 0; i < sz(v); i++) {
			info[v[i]].hind = i;
			info[v[i]].hsz = sz(v);
		}
	}

	auto process = [&](auto cmp) -> void {
		sort(arr, arr + n, [&](const Tree &a, const Tree &b) -> bool {
			int cmpa = cmp(a);
			int cmpb = cmp(b);
			return tie(cmpa, a.y) < tie(cmpb, b.y);
		});
		for(int i = 0; i < n - 1; i++) {
			if(cmp(arr[i]) == cmp(arr[i + 1])) {
				addEdge(i);
			}
		}
	};

	// up
	process([&](const Tree &x) -> int {
		return x.x;
	});

	// up left
	process([&](const Tree &x) -> int {
		return x.x + x.y;
	});

	// up right
	process([&](const Tree &x) -> int {
		return x.x - x.y;
	});

	sort(arr, arr + n, [&](const Tree &a, const Tree &b) -> bool {
		return a.i < b.i;
	});

	memset(memo, -1, sizeof(memo));
	int ans = dp(n - 1) - 1;
	cout << ans << endl;
	dbg(ans);
	int u = n - 1;
	vector<int> xans;
	while(true) {
		int start = nxt[u].tind;
		u = nxt[u].v;
		if(u == -1) {
			break;
		}
		int cu = compr(arr[u].y);
		int end = nxt[u].find;
		dbg(start, end);
		if(end == -1) {
			xans.push_back(hlines[cu][start]);
			for(int i = start - 1; i >= 0; i--) {
				xans.push_back(hlines[cu][i]);
			}
			for(int i = start + 1; i < sz(hlines[cu]); i++) {
				xans.push_back(hlines[cu][i]);
			}
			break;
		}
		xans.push_back(hlines[cu][start]);
		if(start < end) {
			for(int i = start - 1; i >= 0; i--) {
				xans.push_back(hlines[cu][i]);
			}
			for(int i = start + 1; i <= end; i++) {
				xans.push_back(hlines[cu][i]);
			}
		}else if(end < start) {
			for(int i = start + 1; i < sz(hlines[cu]); i++) {
				xans.push_back(hlines[cu][i]);
			}
			for(int i = start - 1; i >= end; i--) {
				xans.push_back(hlines[cu][i]);
			}
		}
	}
	assert(sz(xans) == ans);
	for(auto &a: xans) {
		cout << a + 1 << " ";
	}
	cout << endl;
	gdp(n - 1);
	{
		memset(vis, 0, sizeof(vis));
		for(int i = 0; i < n; i++) {
			dfs(i);
		}
		reverse(begin(order), end(order));
		int cnt[n] {}, vis[n] {};
		int ans = 0, time = 0;
		auto search = [&](int xu, int req) -> void {
			time++;
			queue<int> q;
			q.push(xu);
			while(sz(q)) {
				int u = q.front();
				q.pop();
				if(u != xu && cnt[u]) {
					cnt[u]--;
					cnt[xu]++;
					if(--req <= 0) {
						return;
					}
				}else {
					for(auto &v: igraph[u]) {
						if(vis[v] != time) {
							vis[v] = time;
							q.push(v);
						}
					}
				}
			}
			ans += req;
			cnt[xu] += req;
		};
		for(auto &a: order) {
			if(cnt[a] < sz(ngraph[a])) {
				search(a, sz(ngraph[a]) - cnt[a]);
			}
			for(auto &b: ngraph[a]) {
				cnt[b]++;
				cnt[a]--;
			}
		}
		cout << ans << endl;
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}