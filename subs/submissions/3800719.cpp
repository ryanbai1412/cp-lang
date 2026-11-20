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

//typing too hard
#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 7e4 + 100, maxt = 4 * maxn;

namespace DS {
	const int maxt = 4 * maxn;

	bool rem[maxn];
	set<pair<int, int>> v[maxt];

	void update(int o, int l, int r, int x, int y, int ind) {
		v[o].emplace(y, ind);
		if(l == r) {
			return;
		}
		int mid = (l + r) / 2;
		if(x <= mid) {
			update(o * 2, l, mid, x, y, ind);
		}else {
			update(o * 2 + 1, mid + 1, r, x, y, ind);
		}
	}

	void update(int x, int y, int ind) {
		update(1, 0, maxn, x, y, ind);
	}

	void query(int o, int l, int r, int qlx, int qrx, int qly, int qry, vector<int> &ans) {
		auto it = v[o].lower_bound({qly, 0});
		while(it != v[o].end() && rem[it->second]) {
			it = v[o].erase(it);
		}
		if(it == v[o].end() || it->first > qry) {
			return;
		}
		if(qlx <= l && r <= qrx) {
			while(it != v[o].end() && it->first <= qry) {
				if(!rem[it->second]) {
					rem[it->second] = true;
					ans.push_back(it->second);
				}
				it = v[o].erase(it);
			}
			return;
		}
		int mid = (l + r) / 2;
		if(qlx <= mid) {
			query(o * 2, l, mid, qlx, qrx, qly, qry, ans);
		}
		if(mid < qrx) {
			query(o * 2 + 1, mid + 1, r, qlx, qrx, qly, qry, ans);
		}
	}

	vector<int> query(int qlx, int qrx, int qly, int qry) {
		vector<int> ans;
		query(1, 0, maxn, qlx, qrx, qly, qry, ans);
		return ans;
	}
}

void solve() {
	int n, m, w, h;
	cin >> n >> m >> w >> h;
	for(int i = 0; i < n; i++) {
		int x, y;
		cin >> x >> y;
		DS::update(x, y, i);
	}
	int portals[m][4];
	vector<pair<int, int>> graph[n];
	for(int i = 0; i < m; i++) {
		int u, t;
		cin >> u >> t;
		u--;
		graph[u].emplace_back(i + n, t);
		for(int j = 0; j < 4; j++) {
			cin >> portals[i][j];
		}
	}
	int dist[n + m];
	fill(dist, dist + n + m, INT_MAX);
	mpq<pair<int, int>> pq;
	auto push = [&](int u, int d) -> void {
		if(d < dist[u]) {
			dist[u] = d;
			pq.emplace(d, u);
		}
	};
	push(0, 0);
	while(sz(pq)) {
		auto[d, u] = pq.top();
		pq.pop();
		if(u < n) {
			for(auto &[v, w]: graph[u]) {
				push(v, d + w);
			}
		}else {
			auto &[qlx, qrx, qly, qry] = portals[u - n];
			for(auto &a: DS::query(qlx, qrx, qly, qry)) {
				push(a, d);
			}
		}
	}
	for(int i = 1; i < n; i++) {
		cout << dist[i] << endl;
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