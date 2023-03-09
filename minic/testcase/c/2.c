int int_max(int a, int b)
{
        if (a > b)
                return a;
        return b;
}

int (*get_the_other_buf(int (*buf1)[2], int (*buf2)[2],
        int (*buf)[2], int buf_id, int* other))[2]
{
        if (buf_id == 1) {
                *other = 2;
                return buf2 + (buf - buf1);
        }
        *other = 1;
        return buf1 + (buf - buf2);
}

void skyline_add_key_point(int (*skyline)[2], int* skyline_size,
        int x, int y)
{
        if (*skyline_size > 0) {
                if (skyline[(*skyline_size) - 1][1] == y) {
                        // printf("redundant\n");
                        return;
                }
        }
        int(*key_point)[2] = &skyline[*skyline_size];
        *skyline_size = *skyline_size + 1;
        // printf("old: %d\n", (*key_point)[0]);
        (*key_point)[0] = x;
        // printf("old: %d\n", (*key_point)[1]);
        (*key_point)[1] = y;
        // printf("add %d %d\n", x, y);
}

void building_to_skyline(int* building, int (*skyline)[2], int* skyline_size)
{
        int left = building[0];
        int right = building[1];
        int height = building[2];
        skyline_add_key_point(skyline, skyline_size, left, height);
        skyline_add_key_point(skyline, skyline_size, right, 0);
}

void merge_skyline(int (*skyline1)[2], int skyline1_size, int (*skyline2)[2],
        int skyline2_size, int (*skyline)[2], int* skyline_size)
{
        // printf("merge %d + %d\n", skyline1_size, skyline2_size);
        int i1 = 0;
        int i2 = 0;
        int h1 = 0;
        int h2 = 0;
        while (i1 < skyline1_size && i2 < skyline2_size) {
                int x1 = skyline1[i1][0];
                int y1 = skyline1[i1][1];
                int x2 = skyline2[i2][0];
                int y2 = skyline2[i2][1];
                int x;
                int y;
                int* h;
                if (x1 <= x2) {
                        i1 = i1 + 1;
                        x = x1;
                        y = y1;
                        h1 = y;
                }
                if (x2 <= x1) {
                        i2 = i2 + 1;
                        x = x2;
                        y = y2;
                        h2 = y;
                }
                skyline_add_key_point(skyline, skyline_size, x,
                        int_max(h1, h2));
        }
        for (; i1 < skyline1_size; i1 = i1 + 1) {
                int x = skyline1[i1][0];
                int y = skyline1[i1][1];
                skyline_add_key_point(skyline, skyline_size, x, y);
        }
        for (; i2 < skyline2_size; i2 = i2 + 1) {
                int x = skyline2[i2][0];
                int y = skyline2[i2][1];
                skyline_add_key_point(skyline, skyline_size, x, y);
        }
}

void get_skyline(int** buildings, int l, int r, int (*skyline_buf1)[2],
        int (*skyline_buf2)[2], int (*skyline)[2], int* skyline_size,
        int buf_id)
{
        if (l == r) {
                int* building = buildings[l];
                // printf("add building %d\n", l);
                building_to_skyline(building, skyline, skyline_size);
                return;
        }
        assert(l < r);
        int buildings_size = r - l + 1;
        int m = l + (r - l) / 2;
        int other_buf;
        int(*merge_buf)[2] = get_the_other_buf(skyline_buf1, skyline_buf2,
                skyline, buf_id, &other_buf);
        int(*skyline1)[2] = merge_buf;
        int skyline1_size = 0;
        int(*skyline2)[2] = merge_buf + 2 * (m - l + 1);
        int skyline2_size = 0;
        get_skyline(buildings, l, m, skyline_buf1, skyline_buf2, skyline1,
                &skyline1_size, other_buf);
        // printf("%d %d\n", l, r);
        for (int i = 0; i < skyline1_size; i = i + 1) {
                // printf("%d %d, ", skyline1[i][0], skyline1[i][1]);
        }
        // printf("\n");
        get_skyline(buildings, m + 1, r, skyline_buf1, skyline_buf2, skyline2,
                &skyline2_size, other_buf);
        // printf("%d %d\n", l, r);
        for (int i = 0; i < skyline1_size; i = i + 1) {
                // printf("%d %d, ", skyline1[i][0], skyline1[i][1]);
        }
        // printf("\n");
        merge_skyline(skyline1, skyline1_size, skyline2, skyline2_size, skyline,
                skyline_size);
}

/**
 * Return an array of arrays of size *returnSize.
 * The sizes of the arrays are returned as *returnColumnSizes array.
 * Note: Both returned array and *columnSizes array must be malloced, assume caller calls free().
 */
int** getSkyline(int** buildings, int buildingsSize, int* buildingsColSize,
        int* returnSize, int** returnColumnSizes)
{
        int skyline_size_max = 2 * buildingsSize;
        int skyline_buf1[1024][2];
        int skyline_buf2[1024][2];
        /*
        for (int i = 0; i < skyline_size_max; i++) {
                skyline_buf1[i][0] = 100;
                skyline_buf1[i][1] = 100;
                skyline_buf2[i][0] = 200;
                skyline_buf2[i][1] = 200;
        }
        */

        int(*skyline)[2] = skyline_buf1;
        int skyline_size = 0;

        get_skyline(buildings, 0, buildingsSize - 1, skyline_buf1, skyline_buf2,
                skyline, &skyline_size, 1);

        *returnSize = skyline_size;
        int** ret = malloc(skyline_size * sizeof(*ret));
        *returnColumnSizes = malloc(skyline_size * sizeof(**returnColumnSizes));
        for (int i = 0; i < skyline_size; i = i + 1) {
                (*returnColumnSizes)[i] = 2;
                ret[i] = malloc(2 * sizeof(*ret[i]));
                for (int j = 0; j < 2; j = j + 1)
                        ret[i][j] = skyline[i][j];
        }
        return ret;
}
