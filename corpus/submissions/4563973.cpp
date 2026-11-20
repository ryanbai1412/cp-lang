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
	string a, b;
	cin >> a >> b;
	int n = sz(a), m = sz(b);
	int bptr = 0;
	vector<char> s;
	for (auto& c : a) {
		s.push_back(c);
		while (sz(s) >= 2 && s[sz(s) - 2] == s[sz(s) - 1] && s.back() != 'z') {
			char x = s.back();
			s.pop_back();
			s.pop_back();
			s.push_back(char(x + 1));
		}
		while (bptr < m && sz(s) && s[0] == b[bptr]) {
			s.erase(s.begin());
			bptr++;
		}
	}
	if (!sz(s) && bptr == m) {
		cout << "YES" << endl;
	} else {
		cout << "NO" << endl;
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