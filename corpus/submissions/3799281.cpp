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

const int maxn = 7e4 + 100, mnode = 1e7;

vector<pair<int, int>> graph[mnode];
int dist[mnode];

namespace DS {
	int troot = 1, hind = 2;

	struct Node {
		int o, lc, rc;
	} heap[mnode];

	inline int alloc() {
		return hind++;
	}

	int create1(int &o, int l, int r, int x, int y) {
		if(!o) {
			o = alloc();
		}
		if(l == r) {
			return o;
		}
		int mid = (l + r) / 2;
		if(y <= mid) {
			return create1(heap[o].lc, l, mid, x, y);
		}
		return create1(heap[o].rc, mid + 1, r, x, y);
	}

	int create2(int &o, int l, int r, int x, int y) {
		if(!o) {
			o = alloc();
		}
		if(l == r) {
			return create1(heap[o].o, 0, maxn, x, y);
		}
		int mid = (l + r) / 2;
		if(x <= mid) {
			return create2(heap[o].lc, l, mid, x, y);
		}
		return create2(heap[o].rc, mid + 1, r, x, y);
	}

	int create(int x, int y) {
		return create2(troot, 0, maxn, x, y);
	}

	void update1(int &o, int l, int r, int xl, int xr, int yl, int yr, int v) {
		if(!o) {
			o = alloc();
		}
		if(yl <= l && r <= yr) {
			dbg(o);
			graph[v].emplace_back(o, 0);
			return;
		}
		int mid = (l + r) / 2;
		if(yl <= mid) {
			update1(heap[o].lc, l, mid, xl, xr, yl, yr, v);
		}
		if(mid < yr) {
			update1(heap[o].rc, mid + 1, r, xl, xr, yl, yr, v);
		}
	}

	void update2(int &o, int l, int r, int xl, int xr, int yl, int yr, int v) {
		if(!o) {
			o = alloc();
		}
		if(xl <= l && r <= xr) {
			update1(heap[o].o, 0, maxn, xl, xr, yl, yr, v);
			return;
		}
		int mid = (l + r) / 2;
		if(xl <= mid) {
			update2(heap[o].lc, l, mid, xl, xr, yl, yr, v);
		}
		if(mid < xr) {
			update2(heap[o].rc, mid + 1, r, xl, xr, yl, yr, v);
		}
	}

	void update(int xl, int xr, int yl, int yr, int v) {
		update2(troot, 0, maxn, xl, xr, yl, yr, v);
	}

	vector<int> path;

	void build1(int oa, int ob, int l, int r) {
		if(!(oa && ob)) {
			return;
		}
		graph[oa].emplace_back(ob, 0);
		if(l == r) {
			return;
		}
		int mid = (l + r) / 2;
		build1(heap[oa].lc, heap[ob].lc, l, mid);
		build1(heap[oa].rc, heap[ob].rc, mid + 1, r);
	}

	void build2(int o, int l, int r) {
		if(!o) {
			return;
		}
		if(l == r) {
			for(auto &a: path) {
				build1(heap[a].o, heap[o].o, 0, maxn);
			}
			return;
		}
		path.push_back(o);
		int mid = (l + r) / 2;
		build2(heap[o].lc, l, mid);
		build2(heap[o].rc, mid + 1, r);
		path.pop_back();
	}

	void finalize() {
		build2(troot, 0, maxn);
		for(int i = 0; i < mnode; i++) {
			auto &[_, lc, rc] = heap[i];
			if(lc) {
				graph[i].emplace_back(lc, 0);
			}
			if(rc) {
				graph[i].emplace_back(rc, 0);
			}
		}
	}
}

void solve() {
	int n, m, w, h;
	cin >> n >> m >> w >> h;
	int cities[n];
	for(int i = 0; i < n; i++) {
		int x, y;
		cin >> x >> y;
		cities[i] = DS::create(x, y);
		dbg(i, cities[i]);
	}
	for(int i = 0; i < m; i++) {
		int u, w, xl, xr, yl, yr;
		cin >> u >> w >> xl >> xr >> yl >> yr;
		u--;
		int tmp = DS::alloc();
		graph[cities[u]].emplace_back(tmp, w);
		DS::update(xl, xr, yl, yr, tmp);
	}
	DS::finalize();

	mpq<pair<int, int>> pq;
	fill(begin(dist), end(dist), INT_MAX);
	auto push = [&](int u, int d) -> void {
		if(d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, u);
		}
	};
	push(cities[0], 0);
	while(sz(pq)) {
		auto[d, u] = pq.top();
		pq.pop();
		if(dist[u] == d) {
			for(auto &[v, w]: graph[u]) {
				push(v, d + w);
			}
		}
	}
	for(int i = 1; i < n; i++) {
		dbg(i, dist[cities[i]]);
		assert(dist[cities[i]] != INT_MAX);
		cout << dist[cities[i]] << endl;
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