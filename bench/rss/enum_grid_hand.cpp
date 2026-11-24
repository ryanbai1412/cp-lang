// Matched C++ for enum_grid.cp: one byte per cell.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
enum class Colour : unsigned char { Red, Blue };
int main() {
    ll n;
    if (scanf("%lld", &n) != 1) return 1;
    vector<Colour> grid(n * n, Colour::Red);
    ll x = 12345;
    for (ll i = 0; i < n * n; ++i) {
        x = (x * 1103515245 + 12345) % 2147483647;
        if (x % 3 == 0) grid[i] = Colour::Blue;
    }
    ll blue = 0;
    for (ll i = 0; i < n * n; ++i) {
        x = (x * 1103515245 + 12345) % 2147483647;
        if (grid[x % (n * n)] == Colour::Blue) ++blue;
    }
    printf("%lld\n", blue);
}
