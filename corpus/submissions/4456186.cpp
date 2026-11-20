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

using Mat = array<array<int, 5>, 5>;

Mat matinf() {
	Mat ans;
	for (auto& a : ans) {
		fill(begin(a), end(a), 1e9);
	}
	return ans;
}

Mat operator+(const Mat& x, const Mat& y) {
	Mat ans = matinf();
	for (int i = 0; i < 5; i++) {
		for (int j = 0; j < 5; j++) {
			for (int k = 0; k < 5; k++) {
				ans[i][k] = min(ans[i][k], x[i][j] + y[j][k]);
			}
		}
	}
	return ans;
}

struct ST {
	int n;
	vector<Mat> arr, v;

	ST(const vector<Mat>& arr) : n(sz(arr)), arr(arr), v(4 * n) {
		build(1, 0, n - 1);
	}

	void build(int o, int l, int r) {
		if (l == r) {
			v[o] = arr[l];
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			build(lc, l, mid);
			build(rc, mid + 1, r);
			v[o] = v[lc] + v[rc];
		}
	}

	Mat query(int o, int l, int r, int ql, int qr) const {
		if (ql <= l && r <= qr) {
			return v[o];
		} else {
			int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
			if (ql <= mid) {
				if (mid < qr) {
					return query(lc, l, mid, ql, qr) + query(rc, mid + 1, r, ql, qr);
				}
				return query(lc, l, mid, ql, qr);
			}
			return query(rc, mid + 1, r, ql, qr);
		}
	}

	Mat query(int l, int r) const {
		return query(1, 0, n - 1, l, r);
	}
};

void solve() {
	int k, n, m, q;
	cin >> k >> n >> m >> q;
	vector<Mat> arr(n / k + 1, matinf());
	for (int i = 0; i < m; i++) {
		int u, v, w;
		cin >> u >> v >> w;
		arr[u / k][u % k][v % k] = w;
	}
	ST st(arr);
	while (q--) {
		int u, v;
		cin >> u >> v;
		if (u / k >= v / k) {
			if (u == v) {
				cout << 0 << endl;
			} else {
				cout << -1 << endl;
			}
			continue;
		}
		Mat ans = st.query(u / k, v / k - 1);
		int cans = ans[u % k][v % k];
		if (cans < int(1e9)) {
			cout << cans << endl;
		} else {
			cout << -1 << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}