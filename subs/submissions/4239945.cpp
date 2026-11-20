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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename Node_T, typename Operation = plus<Node_T>, bool debug = true>
class SegTree {
  private:
	static constexpr Node_T noT {};

  public:
	const int n;
	const Operation operation;

	vector<Node_T> v, arr;

	explicit SegTree(const vector<Node_T>& _arr, const Operation& _operation = Operation())
		: n(sz(_arr)), operation(_operation), v(4 * n), arr(_arr) {
		if (n) {
			build(1, 0, n - 1);
		}
	}

  private:
	void build(int o, int l, int r) {
		if (l == r) {
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	template <typename F>
	void set_func(int o, int l, int r, int ind, const F& f) {
		if (l == r) {
			f(arr[l]);
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			if (ind <= mid) {
				set_func(o * 2, l, mid, ind, f);
			} else {
				set_func(o * 2 + 1, mid + 1, r, ind, f);
			}
			v[o] = operation(v[o * 2], v[o * 2 + 1]);
		}
	}

	[[nodiscard]] Node_T query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2;
			if (ql <= mid) {
				if (mid < qr) {
					return operation(query(o * 2, l, mid, ql, qr),
									 query(o * 2 + 1, mid + 1, r, ql, qr));
				}
				return query(o * 2, l, mid, ql, qr);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr);
		}
	}

  public:
	[[nodiscard]] Node_T query(int l, int r) const {
		if (debug) {
			assert(0 <= l);
			assert(l < n);
			assert(0 <= r);
			assert(r < n);
			assert(l <= r);
		}
		return query(1, 0, n - 1, l, r);
	}

	void set(int ind, const Node_T& x) {
		if (debug) {
			assert(0 <= ind);
			assert(ind < n);
		}
		set_func(1, 0, n - 1, ind, [&](Node_T& t) {
			t = x;
		});
	}
};

template <typename T>
struct Min {
	T operator()(const T& a, const T& b) const {
		return min(a, b);
	}
};

const int maxn = 2e5 + 5;

int d, arr[maxn];
long ans[maxn];
vector<int> graph[maxn];
SegTree<long, Min<long>> st((vector<long>(maxn)));

void dfs(int u, int p = -1, int depth = 0) {
	long cur = 0;
	if (depth) {
		cur = st.query(max(0, depth - d), depth - 1) + arr[u];
	}
	ans[u] = cur;
	st.set(depth, cur);
	for (auto& v : graph[u]) {
		if (v != p) {
			dfs(v, u, depth + 1);
		}
	}
}

void solve() {
	int n;
	cin >> n >> d;
	for (int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	for (int i = 0; i < n - 1; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	dfs(0);
	for (int i = 0; i < n; i++) {
		cout << ans[i] << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}