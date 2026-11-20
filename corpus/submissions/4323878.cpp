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

using V = array<array<int, 3>, 3>;

int solve(V vals) {
	int ans = 0;
	for (int i = 0; i < 3; i++) {
		for (int j = i + 1; j < 3; j++) {
			int sub = min(vals[i][j], vals[j][i]);
			ans += sub;
			vals[i][j] -= sub;
			vals[j][i] -= sub;
		}
	}
	ans += 2 * (vals[0][1] + vals[0][2]);
	return ans;
}

int solve(const vector<int>& arr) {
	int n = sz(arr);
	int ca = int(count(begin(arr), end(arr), 0));
	int cb = int(count(begin(arr), end(arr), 1));
	int cc = n - ca - cb;
	int ia = 0, ib = ca, ic = ca + cb;
	int targ[n];
	fill(targ, targ + ib, 0);
	fill(targ + ib, targ + ic, 1);
	fill(targ + ic, targ + n, 2);
	ia %= n;
	ib %= n;
	ic %= n;
	V cur {};
	auto add = [&](int i, int x) -> void {
		cur[arr[i]][targ[i]] += x;
	};
	for (int i = 0; i < n; i++) {
		add(i, 1);
	}
	int ans = 1e9;
	auto inc = [&](int& x) -> void {
		x++;
		if (x >= n) {
			x -= n;
		}
	};
	if (cc) {
		for (int i = 0; i < n; i++) {
			array<int, 3> ord {ia, ib, ic};
			for (auto& a : ord) {
				add(a, -1);
			}
			sort(begin(ord), end(ord));
			swap(targ[ord[2]], targ[ord[1]]);
			swap(targ[ord[1]], targ[ord[0]]);
			for (auto& a : ord) {
				add(a, +1);
			}
			inc(ia);
			inc(ib);
			inc(ic);
			ans = min(ans, solve(cur));
		}
	} else {
		for (int i = 0; i < n; i++) {
			int ord[3] {ia, ib, ic};
			sort(begin(ord), end(ord));
			int e = int(unique(begin(ord), end(ord)) - ord);
			for (int j = 0; j < e; j++) {
				add(ord[j], -1);
			}
			for (int j = e - 1; j >= 1; j--) {
				swap(targ[ord[j]], targ[ord[j - 1]]);
			}
			for (int j = 0; j < e; j++) {
				add(ord[j], +1);
			}
			inc(ia);
			inc(ib);
			inc(ic);
			ans = min(ans, solve(cur));
		}
	}
	return ans;
}

void solve() {
	string s;
	cin >> s;
	int n = sz(s);
	bool ca = s.find('A') == string::npos;
	bool cb = s.find('B') == string::npos;
	bool cc = s.find('C') == string::npos;
	if (ca + cb + cc >= 2) {
		cout << 0 << endl;
		return;
	}
	if (ca) {
		for (auto& a : s) {
			if (a == 'A') {
				a = 'C';
			}
		}
	}
	if (cb) {
		for (auto& a : s) {
			if (a == 'B') {
				a = 'C';
			}
		}
	}
	if (s.find('C') == string::npos) {
		vector<int> arr(n);
		for (int i = 0; i < n; i++) {
			arr[i] = s[i] - 'A';
		}
		cout << solve(arr) << endl;
		return;
	}
	int ans = 1e9;
	int perm[3] {0, 1, 2};
	do {
		vector<int> arr(n);
		for (int i = 0; i < n; i++) {
			arr[i] = perm[s[i] - 'A'];
		}
		ans = min(ans, solve(arr));
	} while (next_permutation(begin(perm), end(perm)));
	cout << ans << endl;
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