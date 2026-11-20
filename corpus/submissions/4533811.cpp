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

using A = array<string, 3>;

namespace s1 {

void solve(const vector<A> &arr) {
	int n = sz(arr);
	pair<int, vector<int>> ans {-1, {}};
	int perm[n];
	iota(perm, perm + n, 0);
	do {
		int cans = 0;
		for (int i = 0; i + 1 < n; i++) {
			if (arr[perm[i]][2] == "loushang" && arr[perm[i]][1] == arr[perm[i + 1]][0]) {
				cans++;
			}
		}
		for (int i = 1; i < n; i++) {
			if (arr[perm[i]][2] == "louxia" && arr[perm[i]][1] == arr[perm[i - 1]][0]) {
				cans++;
			}
		}
		if (cans > ans.first) {
			ans = {cans, vector<int> (perm, perm + n)};
		}
	} while (next_permutation(perm, perm + n));
	cout << ans.first << endl;
	for (int i = 0; i < n; i++) {
		cout << ans.second[i] + 1 << " \n"[i == n - 1];
	}
}

}

void solve() {
	int n, m;
	cin >> n >> m;
	for (int i = 0; i < n; i++) {
		string s;
		cin >> s;
	}
	vector<A> arr(m);
	for (auto& [a, b, c] : arr) {
		cin >> a >> b >> c;
	}
	if (m <= 10) {
		s1::solve(arr);
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int tcs;
	cin >> tcs;
	while (tcs--) {
		solve();
	}
}