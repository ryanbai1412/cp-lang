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

vector<int> solve(vector<int> arr) {
	vector<int> ans;
	set<int> vals;
	for (int i = 0; i < sz(arr); i++) {
		if (arr[i] == 2) {
			vals.insert(i);
		}
	}
	auto toggle = [&](int i) -> void {
		if (0 <= i && i < sz(arr) && arr[i]) {
			if (arr[i] == 2) {
				vals.erase(i);
			} else {
				vals.insert(i);
			}
			arr[i] = 3 - arr[i];
		}
	};
	while (sz(vals)) {
		int u = *vals.begin();
		vals.erase(vals.begin());
		arr[u] = 0;
		ans.push_back(u);
		toggle(u - 1);
		toggle(u + 1);
	}
	return ans;
}

void solve() {
	int n, m;
	cin >> n >> m;
	int arr[n][m];
	for (auto& a : arr) {
		string s;
		cin >> s;
		for (int i = 0; i < m; i++) {
			a[i] = s[i] == 'H' ? 2 : 1;
		}
	}
	vector<int> hori(n);
	for (int i = 0; i < n; i++) {
		hori[i] = count(arr[i], arr[i] + m, 2) & 1 ? 2 : 1;
	}
	auto hord = solve(hori);
	vector<pair<int, int>> ans;
	for (auto& a : hord) {
		dbg(a);
		vector<int> cur(arr[a], arr[a] + m);
		if (hori[a] == 1) {
			for (auto& b : cur) {
				b = 3 - b;
			}
		}
		auto vord = solve(cur);
		for (auto& b : vord) {
			ans.emplace_back(a, b);
		}
	}
	if (sz(ans) == n * m) {
		for (auto& [x, y] : ans) {
			cout << x + 1 << " " << y + 1 << endl;
		}
	} else {
		cout << -1 << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}