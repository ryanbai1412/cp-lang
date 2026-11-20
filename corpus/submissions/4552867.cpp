#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
	cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
		 << ": ";                                          \
	dbgh(__VA_ARGS__)
#else
#define cerr   \
	if (false) \
	cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

namespace __gnu_pbds {
template <typename T>
using ost = tree<T, null_type, less<>, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

const int maxn = 8e5 + 5;

struct Bit {
	int n;
	vector<int> v;

	Bit(int n) : n(n), v(n + 1) {}

	void update(int ind, int x) {
		ind++;
		while (ind <= n) {
			v[ind] += x;
			ind += ind & -ind;
		}
	}

	int query(int ind) const {
		int ans = 0;
		ind++;
		while (ind > 0) {
			ans += v[ind];
			ind -= ind & -ind;
		}
		return ans;
	}
};

vector<int> graph[maxn], times, tour;

void ptour(int t, int u) {
	times.push_back(t);
	tour.push_back(u);
};

void dfs(int u, int p = -1, int t = 0) {
	int n = sz(graph[u]);
	int ts[n], ind = -1;
	for (int i = 0; i < n; i++) {
		if (graph[u][i] == p) {
			t++;
			ind = i;
			continue;
		}
		ts[i] = t;
	}
	if (ind != -1) {
		rotate(ts, ts + ind, ts + n);
		rotate(graph[u].begin(), graph[u].begin() + ind, graph[u].end());
	}
	for (int i = ind != -1; i < n; i++) {
		t = ts[i];
		int v = graph[u][i];
		ptour(t, v);
		dfs(v, u, t);
		ptour(t, u);
	}
}

void solve() {
	int n, q;
	cin >> n >> q;
	for (int i = 0; i < n; i++) {
		int m;
		cin >> m;
		graph[i].resize(m);
		for (auto& a : graph[i]) {
			cin >> a;
			a--;
		}
		graph[i].push_back(graph[i][0]);
		graph[i].erase(graph[i].begin());
	}
	dfs(0);
	int cnt[n + 1] {};
	for (auto& a : times) {
		cnt[a]++;
	}
	long psum1[n + 1];
	for (int i = 0; i <= n; i++) {
		psum1[i] = (i ? psum1[i - 1] : 0) + cnt[i];
	}
	long psum[n + 1];
	for (int i = 0; i <= n; i++) {
		psum[i] = (i ? psum[i - 1] : 0) + psum1[i];
	}
	vector<pair<int, int>> queries[n + 2];
	for (int i = 0; i < q; i++) {
		long x;
		cin >> x;
		int ind = int(lower_bound(psum, psum + n + 1, x) - psum);
		if (ind) {
			x -= psum[ind - 1];
		}
		if (ind > n) {
			x %= 2 * (n - 1);
		}
		queries[ind].emplace_back(x, i);
	}
	int ans[q];
	vector<int> inds[n + 2];
	for (int i = 0; i < sz(times); i++) {
		inds[times[i]].push_back(i);
	}
	ost<pair<int, int>> vals;
	for (int i = 0; i <= n + 1; i++) {
		for (auto& a : inds[i]) {
			vals.insert({a, tour[a]});
		}
		for (auto& [x, qi] : queries[i]) {
			if (!x) {
				ans[qi] = 0;
				continue;
			}
			ans[qi] = vals.find_by_order(x - 1)->second;
		}
	}
	for (auto& a : ans) {
		cout << a + 1 << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}