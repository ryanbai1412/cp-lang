#include <cstdio>
#include <string>
#include <tuple>
#include <vector>
int find(int n);
std::tuple<int, std::vector<int>> stats(int scale);
static int hidden;
int compare(int x) { return x < hidden ? -1 : x > hidden; }
void note(std::vector<int> tries, std::string label) { printf("%s after %zu\n", label.c_str(), tries.size()); }
int main() {
    for (int h : {1, 37, 100}) { hidden = h; printf("%d\n", find(100)); }
    auto [total, both] = stats(2);
    printf("%d %d %d\n", total, both[0], both[1]);
}
