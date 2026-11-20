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

template <typename T>
struct ST {
	int n;
	vector<T> v, arr;

	ST(const vector<T>& _arr) : n(sz(_arr)), v(4 * n), arr(_arr) {
		if (n) {
			build(1, 0, n - 1);
		}
	}

	void build(int o, int l, int r) {
		if (l == r) {
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2;
			build(o * 2, l, mid);
			build(o * 2 + 1, mid + 1, r);
			v[o] = min(v[o * 2], v[o * 2 + 1]);
		}
	}

	void update(int o, int l, int r, int ind, const T& x) {
		if (l == r) {
			v[o] = x;
		} else {
			int mid = (l + r) / 2;
			if (ind <= mid) {
				update(o * 2, l, mid, ind, x);
			} else {
				update(o * 2 + 1, mid + 1, r, ind, x);
			}
			v[o] = min(v[o * 2], v[o * 2 + 1]);
		}
	}

	void update(int ind, const T& x) { update(1, 0, n - 1, ind, x); }

	T query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2;
			if (ql <= mid) {
				if (mid < qr) {
					return min(query(o * 2, l, mid, ql, qr),
							   query(o * 2 + 1, mid + 1, r, ql, qr));
				}
				return query(o * 2, l, mid, ql, qr);
			}
			return query(o * 2 + 1, mid + 1, r, ql, qr);
		}
	}

	T query(int l, int r) const { return query(1, 0, n - 1, l, r); }
};

const int maxn = 1e6 + 5;

// minimal value with time < query
struct DS {
	struct iterator {
		int val, time, ind;

		bool exists() const { return ind < 1e9; }
	} it_end {0, 0, int(1e9)};

	static vector<pair<int, int>> get_vals(const vector<pair<int, int>>& data) {
		vector<pair<int, int>> ans(sz(data));
		for (int i = 0; i < sz(data); i++) {
			ans[i] = {data[i].first, i};
		}
		return ans;
	}

	static vector<int> get_times(const vector<pair<int, int>>& data) {
		vector<int> ans(sz(data));
		for (int i = 0; i < sz(data); i++) {
			ans[i] = data[i].second;
		}
		return ans;
	}

	ST<pair<int, int>> st;
	vector<int> times;

	DS(const vector<pair<int, int>>& data)
		: DS(get_vals(data), get_times(data)) {}

	DS(const vector<pair<int, int>>& vals, const vector<int>& _times)
		: st(vals), times(_times) {}

	iterator query(int time) const {
		int max_index =
			upper_bound(begin(times), end(times), time) - times.begin();
		if (!max_index) {
			return it_end;
		}
		auto [val, index] = st.query(0, max_index - 1);
		if (val == 1e9) {
			return it_end;
		}
		return {val, times[index], index};
	}

	void erase(const iterator& it) { st.update(it.ind, {1e9, 0}); }
};

vector<DS> dvals;
vector<int> cur;

bool dfs(int ind, int pval, int ptime) {
	if (!ind) {
		return true;
	}
	while (true) {
		auto it = dvals[ind].query(ptime);
		if (!it.exists() || it.val >= pval) {
			return false;
		}
		dvals[ind].erase(it);
		cur.push_back(it.time);
		if (dfs(ind - 1, it.val, it.time)) {
			return true;
		}
		cur.pop_back();
	}
}

void solve() {
	int n;
	cin >> n;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	vector<pair<int, int>> gvals[n + 1];
	int g[n + 1];
	memset(g, 0x3f, sizeof(g));
	g[0] = -1e9;
	for (int i = 0; i < n; i++) {
		int cind = lower_bound(g, g + n + 1, arr[i]) - g;
		g[cind] = arr[i];
		gvals[cind].emplace_back(arr[i], i);
	}
	int mlen = n;
	for (int i = 0; i <= n; i++) {
		if (i && !sz(gvals[i])) {
			mlen = i - 1;
			break;
		}
		dvals.emplace_back(gvals[i]);
	}
	dbg(mlen);
	vector<vector<int>> ans;
	while (true) {
		if (!dfs(mlen, 1e9, 1e9)) {
			break;
		}
		reverse(begin(cur), end(cur));
		ans.push_back(cur);
		cur.clear();
	}
	cout << sz(ans) << " " << mlen << endl;
	for (auto& a : ans) {
		for (int i = 0; i < mlen; i++) {
			cout << a[i] + 1 << " \n"[i == mlen - 1];
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}