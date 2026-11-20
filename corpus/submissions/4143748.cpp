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

struct LST {
	int n;
	vector<int> v, lazy;

	explicit LST(int n) : n(n), v(4 * n), lazy(4 * n) {}

	void update(int o, int l, int r, int ql, int qr, int x) {
		if (ql <= l && r <= qr) {
			lazy[o] += x;
			v[o] += x;
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ql <= mid) {
				update(lc, l, mid, ql, qr, x);
			}
			if (mid < qr) {
				update(rc, mid + 1, r, ql, qr, x);
			}
			v[o] = lazy[o] + max(v[lc], v[rc]);
		}
	}

	void update(int l, int r, int x) {
		update(1, 0, n - 1, l, r, x);
	}

	int query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 0;
			if (ql <= mid) {
				ans = max(ans, query(lc, l, mid, ql, qr));
			}
			if (mid < qr) {
				ans = max(ans, query(rc, mid + 1, r, ql, qr));
			}
			return ans + lazy[o];
		}
	}

	int query(int l, int r) const {
		return query(1, 0, n - 1, l, r);
	}
	
	int query_all() const {
		return v[1];
	}
};

const int maxn = 2e5, logn = 18;

LST st(maxn * 2);

void solve() {
	int n;
	cin >> n;
	int arr[n][2];
	for (auto& [l, r] : arr) {
		cin >> l >> r;
	}
	vector<int> comp;
	for (auto& [l, r] : arr) {
		comp.push_back(l);
		comp.push_back(r);
	}
	sort(begin(comp), end(comp));
	comp.erase(unique(begin(comp), end(comp)), comp.end());
	for (auto& [l, r] : arr) {
		l = int(lower_bound(begin(comp), end(comp), l) - comp.begin());
		r = int(lower_bound(begin(comp), end(comp), r) - comp.begin());
	}
	auto add = [&](int i) -> void {
		if (i < n) {
			st.update(arr[i][0], arr[i][1], 1);
		}
	};
	auto rm = [&](int i) -> void {
		st.update(arr[i][0], arr[i][1], -1);
	};
	int nxt[n];
	{
		int j = 0;
		add(0);
		for (int i = 0; i < n; i++) {
			while (j < n && st.query_all() <= 1) {
				add(++j);
			}
			nxt[i] = j;
			rm(i);
		}
	}
	int lift[logn][n + 1];
	memcpy(lift[0], nxt, sizeof(nxt));
	for (int i = 1; i < logn; i++) {
		lift[i - 1][n] = n;
		for (int j = 0; j < n; j++) {
			lift[i][j] = lift[i - 1][lift[i - 1][j]];
		}
	}
	int q;
	cin >> q;
	while (q--) {
		int l, r;
		cin >> l >> r;
		l--;
		r--;
		int ans = 0;
		for (int i = logn - 1; i >= 0; i--) {
			if (lift[i][l] <= r) {
				ans |= (1 << i);
				l = lift[i][l];
			}
		}
		cout << ans + 1 << endl;
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