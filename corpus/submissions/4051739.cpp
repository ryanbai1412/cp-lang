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

const int maxn = 1e4 + 5, bsize = 100;

int cnt[maxn], cmax;
set<int> ans[maxn];

void init(int n) {
	for (int i = 0; i < n; i++) {
		ans[0].insert(i);
	}
}

void add(int x) {
	ans[cnt[x]++].erase(x);
	ans[cnt[x]].insert(x);
	cmax = max(cmax, cnt[x]);
}

void rm(int x) {
	if (cnt[x] == cmax && sz(ans[cnt[x]]) == 1) {
		cmax--;
	}
	ans[cnt[x]--].erase(x);
	ans[cnt[x]].insert(x);
}

int answer() {
	return *(--ans[cmax].end());
}

struct Query {
	int qind, l, r, lblock;

	Query(int qind, int l, int r) : qind(qind), l(l), r(r), lblock(l / bsize) {}

	bool operator<(const Query& q) const {
		if (lblock == q.lblock) {
			if (lblock & 1) {
				return r > q.r;
			} else {
				return r < q.r;
			}
		} else {
			return lblock < q.lblock;
		}
	}
};

bool intersects(int l1, int r1, int l2, int r2) {
	return max(l1, l2) <= min(r1, r2);
}

void solve() {
	int n, q;
	cin >> n >> q;
	int arr[n];
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	vector<Query> queries;
	for (int i = 0; i < q; i++) {
		int l, r;
		cin >> l >> r;
		l--;
		r--;
		queries.emplace_back(i, l, r);
	}
	sort(queries.begin(), queries.end());
	int ans[q];
	int l = 0, r = -1;
	init(n);
	for (auto& [i, cl, cr, _] : queries) {
		if (intersects(l, r, cl, cr)) {
			for (int i = l; i < cl; i++) {
				rm(arr[i]);
			}
			for (int i = cl; i < l; i++) {
				add(arr[i]);
			}
			for (int i = cr + 1; i <= r; i++) {
				rm(arr[i]);
			}
			for (int i = r + 1; i <= cr; i++) {
				add(arr[i]);
			}
		} else {
			for (int i = l; i <= r; i++) {
				rm(arr[i]);
			}
			for (int i = cl; i <= cr; i++) {
				add(arr[i]);
			}
		}
		ans[i] = answer();
		tie(l, r) = tie(cl, cr);
	}
	for (auto& a : ans) {
		cout << a + 1 << endl;
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