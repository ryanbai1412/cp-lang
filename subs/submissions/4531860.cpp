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

void solve() {
	int n;
	string s;
	cin >> n >> s;
	pair<int, int> arr[n];
	for (auto& [x, y] : arr) {
		cin >> x >> y;
		x--;
		y--;
	}
	bool picked[2 * n] {};
	int ind[2 * n];
	memset(ind, -1, sizeof(ind));
	for (int i = 0; i < n; i++) {
		auto& [x, y] = arr[i];
		if (s[x] == s[y]) {
			picked[x] = s[x] == '(';
			picked[y] = s[y] == ')';
		} else {
			ind[x] = i;
			picked[x] = s[x] == ')';
			picked[y] = s[y] == ')';
			if (s[x] == ')') {
				swap(x, y);
			}
		}
	}
	int csum = 0;
	map<int, int> vals;
	for (int i = 0; i < 2 * n; i++) {
		if (ind[i] != -1) {
			auto& [x, y] = arr[ind[i]];
			vals[max(x, y)] = ind[i];
		}
		if (picked[i]) {
			if (s[i] == '(') {
				csum++;
			} else {
				csum--;
			}
			if (csum < 0) {
				if (!sz(vals)) {
					cout << -1 << endl;
					return;
				}
				auto it = vals.begin();
				auto& [x, y] = arr[it->second];
				if (x <= i) {
					csum++;
				}
				if (y <= i) {
					csum++;
				}
				swap(picked[x], picked[y]);
				vals.erase(it);
			}
		}
	}
	int cur = 0;
	for (int i = 0; i < 2 * n; i++) {
		dbg(i, picked[i]);
		if (picked[i]) {
			if (s[i] == '(') {
				cur++;
			} else {
				cur--;
			}
		}
		assert(cur >= 0);
	}
	if (cur) {
		cout << -1 << endl;
		return;
	}
	cout << "YES" << endl;
	return;
	for (int i = 0; i < n; i++) {
		cout << picked[arr[i].second] << " \n"[i == n - 1];
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