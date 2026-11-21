#include <bits/stdc++.h>
using namespace std;
#include "_pushrelabel.inc"
int main() {
    const int R = 400, C = 400; head.assign(R * C + 2, {});
    auto cell = [&](int r, int c) { return 2 + r * C + c; };
    for (int r = 0; r < R; r++) for (int c = 0; c < C; c++) {
        add(r ? cell(r - 1, c) : 0, cell(r, c), 1);
        add(cell(r, c), c ? cell(r, c - 1) : 1, 1);
    }
    printf("%lld\n", push_relabel(0, 1));
}
