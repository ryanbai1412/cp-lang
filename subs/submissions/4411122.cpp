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

int dist(char ca, int b) {
	int a = ca - 'a';
	return min((a + 26 - b) % 26, (b + 26 - a) % 26);
}

string ts(vector<int> c) {
	string ans;
	for (auto& a : c) {
		ans += char('a' + a);
	}
	return ans;
}

void solve() {
	int k;
	cin >> k;
	string s;
	cin >> s;
	vector<string> ans;
	for (int a = 0; a < 26; a++) {
		for (int b = 0; b < 26; b++) {
			for (int c = 0; c < 26; c++) {
				for (int d = 0; d < 26; d++) {
					int cur = dist(s[0], a) + dist(s[1], b) + dist(s[2], c) + dist(s[3], d);
					if (cur <= k) {
						ans.push_back(ts({a, b, c, d}));
					}
				}
			}
		}
	}
	sort(begin(ans), end(ans));
	for (auto& a : ans) {
		cout << a << endl;
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}