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

struct DS {
	vector<int> perm, rev;

	DS(int n) : perm(n), rev(n) {
		iota(begin(perm), end(perm), 0);
		iota(begin(rev), end(rev), 0);
	}

	void swap_pos(int i, int j) {
		swap(perm[i], perm[j]);
		swap(rev[perm[i]], rev[perm[j]]);
	}

	void swap_num(int i, int j) { swap_pos(rev[i], rev[j]); }
};

void solve() {
	int n, q;
	cin >> n >> q;
	DS arr(n);
	vector<pair<int, int>> b, e;
	while(q--) {
		char type;
		cin >> type;
		if (type == 'B' || type == 'E') {
			int x, y;
			cin >> x >> y;
			x--;
			y--;
			if (type == 'B') {
				b.emplace_back(x, y);
			} else {
				e.emplace_back(x, y);
			}
		} else {
			for (auto& [u, v] : b) {
				arr.swap_num(u, v);
			}
			for (auto& [u, v] : e) {
				arr.swap_pos(u, v);
			}
			b.clear();
			e.clear();
			int from[n];
			for (int i = 0; i < n; i++) {
				from[arr.rev[i]] = i;
			}
			int ans[n];
			for (int i = 0; i < n; i++) {
				int x;
				cin >> x;
				x--;
				ans[from[i]] = x;
			}
			for (int i = 0; i < n; i++) {
				if (i) {
					cout << " ";
				}
				cout << ans[i] + 1;
			}
			cout << endl;
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
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