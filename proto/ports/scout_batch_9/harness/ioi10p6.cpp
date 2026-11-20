// The original is a grader-style submission (it defines LocateCentre, not
// main). This reads the port's input format and calls it.
#include "../../../../corpus/submissions/4843169.cpp"

int main() {
    int n;
    scanf("%d", &n);
    vector<int> fans(n), us(n - 1), vs(n - 1);
    for (auto& a : fans) scanf("%d", &a);
    for (int i = 0; i < n - 1; i++) scanf("%d %d", &us[i], &vs[i]);
    printf("%d\n", LocateCentre(n, fans.data(), us.data(), vs.data()));
}
