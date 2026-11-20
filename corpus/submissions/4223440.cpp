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

void solve() {
	int n, t;
	cin >> n >> t;
	vector<int> targets[n];
	int last[n];
	for (int i = 0; i < n; i++) {
		int x;
		cin >> x;
		targets[i].push_back(x - 1);
	}
	for (int ct = 0; ct < t; ct++) {
		vector<pair<int, int>> upds;
		for (int i = 0; i < n; i++) {
			if (ct < sz(targets[i])) {
				int c = targets[i][ct];
				dbg(i, "throw", c);
				last[i] = c;
				upds.emplace_back(c, i);
			} else {
				targets[i].push_back(0);
				assert(ct < sz(targets[i]));
			}
		}
		for (auto& [a, b] : upds) {
			targets[a].push_back(b);
		}
	}
	for (int i = 0; i < n; i++) {
		cout << last[i] + 1 << " \n"[i == n - 1];
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