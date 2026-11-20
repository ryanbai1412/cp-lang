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

const int maxn = 5e3;

int p[maxn + 1];

void f(int l, int r) {
	if (l >= r)
		return;
	int i = min_element(p + l, p + r + 1) -
			p;  // position of minimum element in [l,r]
	rotate(p + l, p + i,
		   p + i + 1);  // cyclically shift [l,i] to the right by 1, so that the
						// minimum element gets moved to position l
	f(i + 1, r);        // continue on [i+1,r]
}

void solve() {
	int n, q;
	cin >> n >> q;
	for (int i = 1; i <= n; i++) {
		cin >> p[i];
	}
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			f(1, n);
		} else if (t == 2) {
			int ind;
			cin >> ind;
			cout << p[ind] << endl;
		} else {
			int x;
			cin >> x;
			cout << find(begin(p), end(p), x) - p << endl;
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