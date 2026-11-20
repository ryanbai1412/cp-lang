#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, b, d, _m;
	cin >> b >> n >> d >> _m;
	long ans = 0;
	if(b == 1) {
		int arr[n];
		for(int i = 0; i < n; i++) {
			cin >> arr[i];
		}
		sort(arr, arr + n);
		int j = 0;
		for(int i = 0; i < n; i++) {
			while(arr[i] - arr[j] > d) {
				j++;
			}
			ans += i - j;
		}
	}else if(b == 2) {
		const int am = 80000;
		const int m = 80000 * 2;
		pair<int, int> arr[n];
		for(int i = 0; i < n; i++) {
			int x, y;
			cin >> x >> y;
			arr[i].first = x + y;
			arr[i].second = x - y + am;
		}
		sort(arr, arr + n);
		int bit[m + 1] {};
		auto add = [&](int ind, int x) {
			ind++;
			while(ind <= m) {
				bit[ind] += x;
				ind += ind & -ind;
			}
		};
		auto psum = [&](int ind) {
			int ret = 0;
			while(ind > 0) {
				ret += bit[ind];
				ind -= ind & -ind;
			}
			return ret;
		};
		auto sum = [&](int l, int r) {
			return psum(min(m, r + 1)) - psum(max(0, l));
		};
		int j = 0;
		for(int i = 0; i < n; i++) {
			while(arr[i].first - arr[j].first > d) {
				add(arr[j].second, -1);
				j++;
			}
			ans += sum(arr[i].second - d, arr[i].second + d);
			add(arr[i].second, 1);
		}
	}else {
		const int am = 80;
		const int m = 80 * 3;
		array<int, 4> arr[n];
		for(int i = 0; i < n; i++) {
			int x, y, z;
			cin >> x >> y >> z;
			arr[i][0] = x + y + z;
			arr[i][1] = x + y - z + am;
			arr[i][2] = x - y + z + am;
			arr[i][3] = x - y - z + am + am;
		}
		sort(arr, arr + n);
		int bit[m + 1][m + 1][m + 1] {};
		auto add = [&](int _a, int _b, int _c, int x) {
			_a++;
			_b++;
			_c++;
			int a = _a;
			while(a <= m) {
				int b = _b;
				while(b <= m) {
					int c = _c;
					while(c <= m) {
						bit[a][b][c] += x;
						c += c & -c;
					}
					b += b & -b;
				}
				a += a & -a;
			}
		};
		auto psum = [&](int _a, int _b, int _c) {
			int ret = 0;
			int a = _a;
			while(a > 0) {
				int b = _b;
				while(b > 0) {
					int c = _c;
					while(c > 0) {
						ret += bit[a][b][c];
						c -= c & -c;
					}
					b -= b & -b;
				}
				a -= a & -a;
			}
			return ret;
		};
		auto sum = [&](int x1, int x2, int y1, int y2, int z1, int z2) {
			x1 = max(0, x1);
			y1 = max(0, y1);
			z1 = max(0, z1);
			x2 = min(m, x2 + 1);
			y2 = min(m, y2 + 1);
			z2 = min(m, z2 + 1);
			return +psum(x2, y2, z2)
				   - psum(x2, y2, z1)
				   - psum(x2, y1, z2)
				   - psum(x1, y2, z2)
				   + psum(x2, y1, z1)
				   + psum(x1, y2, z1)
				   + psum(x1, y1, z2)
				   - psum(x1, y1, z1);
		};
		int j = 0;
		for(int i = 0; i < n; i++) {
			while(arr[i][0] - arr[j][0] > d) {
				add(arr[j][1], arr[j][2], arr[j][3], -1);
				j++;
			}
			ans += sum(arr[i][1] - d, arr[i][1] + d, arr[i][2] - d, arr[i][2] + d, arr[i][3] - d, arr[i][3] + d);
			add(arr[i][1], arr[i][2], arr[i][3], 1);
		}
	}
	cout << ans << endl;
}