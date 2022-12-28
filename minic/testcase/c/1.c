int remove_duplicates(int* nums, int numsSize)
{
    if (numsSize < 2) {
        return numsSize;
    }
    int* p = nums + 2;
    int* end = nums + numsSize;
    for (int* i = nums + 2; i != end; i = i + 1) {
        if (*(p - 2) < *i) {
            *p = *i;
            p = p + 1;
        }
    }
    return p - nums;
}
