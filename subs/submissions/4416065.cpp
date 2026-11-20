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
	int n, q;
	cin >> n >> q;
	auto sti = [&](const string &s) -> int {
		if (s == "square") {
			return 1;
		} else if (s == "circle") {
			return 2;
		}
		return 3;
	};
	int arr[n] {};
	while (q--) {
		string s, t;
		int i;
		cin >> s >> t >> i;
		i--;
		if (s == "set") {
			arr[i] = sti(t);
		} else {
			cout << (arr[i] == sti(t)) << endl;
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}