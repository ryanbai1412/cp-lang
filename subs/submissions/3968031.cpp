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

struct DS {
	vector<int> perm, rev;

	DS(const vector<int>& arr) : perm(arr), rev(sz(arr)) {
		for (int i = 0; i < sz(arr); i++) {
			rev[arr[i]] = i;
		}
	}

	void swap(int i, int j) {
		std::swap(perm[i], perm[j]);
		std::swap(rev[perm[i]], rev[perm[j]]);
	}
};

int n;

void f(DS &ds) {
	vector<int> change;
	int smin = 1e9;
	for (int i = n - 1; i >= 0; i--) {
		int cur = ds.perm[i];
		if (cur < smin) {
			smin = cur;
			change.push_back(i);
		}
	}
	reverse(begin(change), end(change));
	int prev = 0;
	for (auto& a : change) {
		for (int i = a - 1; i >= prev; i--) {
			ds.swap(i, i + 1);
		}
		prev = a + 1;
	}
}

void solve() {
	int q;
	cin >> n >> q;
	vector<int> arr(n);
	for (auto& a : arr) {
		cin >> a;
		a--;
	}
	DS ds(arr);
	int ucnt = 0;
	while (q--) {
		int t;
		cin >> t;
		if (t == 1) {
			if (++ucnt <= n) {
				f(ds);
			}
		} else if (t == 2) {
			int ind;
			cin >> ind;
			ind--;
			cout << ds.perm[ind] + 1 << endl;
		} else {
			int x;
			cin >> x;
			x--;
			cout << ds.rev[x] + 1 << endl;
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