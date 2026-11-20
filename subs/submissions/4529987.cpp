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

int rk(string name, vector<pair<int, string>> arr) {
	sort(begin(arr), end(arr));
	for (int i = 0; i < sz(arr); i++) {
		if (arr[i].second == name) {
			return i;
		}
	}
	assert(false);
}

void solve() {
	int n;
	cin >> n;
	vector<pair<int, string>> arr(n);
	for (auto& [x, s] : arr) {
		cin >> s;
		x = 0;
		for (int i = 0; i < 5; i++) {
			int a;
			cin >> a;
			x -= a;
		}
	}
	for (int i = 0; i < n; i++) {
		auto tmp = arr;
		tmp[i].first -= 500;
		int a = rk(tmp[i].second, tmp);
		tmp[i].first += 1000;
		int b = rk(tmp[i].second, tmp);
		cout << a + 1 << " " << b + 1 << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}