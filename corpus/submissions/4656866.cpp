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

const int maxn = 2e5 + 5;

struct DSU {
	struct Change {
		bool prevb;
		int u, v, prevpu;
	};

	bool b;
	vector<int> p;
	vector<bool> parity;
	vector<Change> changes;

	DSU(int n) : b(true), p(n, -1), parity(n), changes {} {}

	pair<int, bool> find(int u) const {
		if (p[u] < 0) {
			return {u, false};
		} else {
			auto ans = find(p[u]);
			ans.second ^= parity[u];
			return ans;
		}
	}

	void merge(int qu, int qv) {
		auto [u, pu] = find(qu);
		auto [v, pv] = find(qv);
		if (u == v) {
			changes.push_back({b, -1, -1, -1});
			if (pu == pv) {
				b = false;
			}
			return;
		}
		if (p[u] < p[v]) {
			swap(u, v);
			swap(pu, pv);
		}
		changes.push_back({b, u, v, p[u]});
		p[v] += p[u];
		p[u] = v;
		parity[u] = !(pu ^ pv);
	}

	void rollback() {
		auto [prevb, u, v, prevpu] = changes.back();
		changes.pop_back();
		b = prevb;
		if (u == -1) {
			return;
		}
		parity[u] = false;
		p[u] = prevpu;
		p[v] -= p[u];
	}
};

DSU dsu(maxn);
int ans[maxn], edges[maxn][2];

void push(int i) {
	auto &[u, v] = edges[i];
	dsu.merge(u, v);
}

void pop() {
	dsu.rollback();
}

void dfs(int l, int r, int ql, int qr) {
	if (l > r) {
		return;
	}
	assert(ql <= qr);
	int mid = (l + r) / 2;
	for (int i = l; i < mid; i++) {
		push(i);
	}
	int cr;
	for (cr = qr; cr >= ql; cr--) {
		if (!dsu.b) {
			break;
		}
		push(cr);
	}
	assert(!dsu.b);
	ans[mid] = cr;
	for (int i = cr + 1; i <= qr; i++) {
		pop();
	}
	push(mid);
	dfs(mid + 1, r, cr, qr);
	for (int i = l; i <= mid; i++) {
		pop();
	}
	for (int i = cr + 1; i <= qr; i++) {
		push(i);
	}
	dfs(l, mid - 1, ql, cr);
	for (int i = cr + 1; i <= qr; i++) {
		pop();
	}
}

void solve() {
	int n, m, q;
	cin >> n >> m >> q;
	for (int i = 0; i < m; i++) {
		cin >> edges[i][0] >> edges[i][1];
		edges[i][0]--;
		edges[i][1]--;
	}
	DSU d(n);
	for (int i = 0; i < m; i++) {
		auto &[u, v] = edges[i];
		d.merge(u, v);
	}
	if (d.b) {
		while (q--) {
			cout << "NO" << endl;
		}
		return;
	}
	dfs(0, m - 1, 0, m - 1);
	while (q--) {
		int l, r;
		cin >> l >> r;
		l--;
		r--;
		if (r <= ans[l]) {
			cout << "YES" << endl;
		} else {
			cout << "NO" << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}