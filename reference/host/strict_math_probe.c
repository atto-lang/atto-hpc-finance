/* Host-only negative demonstration, NOT Oreslang-generated code. */
#include <stdio.h>
int main(void) {
    volatile double upper = 1.0e308;
    volatile double lower = -1.0e308;
    double score = upper - lower;
    int approved_side = ((score - score) == 0.0) && (score > 0.0);
    printf("%d\n", approved_side);
    return 0;
}
