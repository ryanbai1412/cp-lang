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
#define dbg(...)                                              \
	cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
	dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
	if (false) \
	cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

void solve() {
	const int n = 100;
	int arr[n][n];
	memset(arr, -1, sizeof(arr));
	arr[0][1] = 1;
	arr[0][2] = 1;
	arr[0][3] = 1;
	arr[1][2] = 1;
	arr[1][3] = 100;
	arr[2][3] = 100;
	for (int i = 2; i + 3 < n; i += 2) {
		arr[i][i + 1] = 100;
		arr[i][i + 2] = 100;
		arr[i][i + 3] = 1;
		arr[i + 1][i + 2] = 1;
		arr[i + 1][i + 3] = 100;
		arr[i + 2][i + 3] = 100;
	}
	cout << n << endl;
	for (int i = 0; i < n; i++) {
		for (int j = i + 1; j < n; j++) {
			if (arr[i][j] == -1) {
				arr[i][j] = 100;
			}
			cout << arr[i][j] << " \n"[j == n - 1];
		}
	}
}

int main() {
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	ios_base::sync_with_stdio(false);
	solve();
}