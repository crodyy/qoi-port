#define STB_IMAGE_IMPLEMENTATION
#define STBI_ONLY_PNG
#define STBI_NO_LINEAR
#include "stb_image.h"

#define QOI_IMPLEMENTATION
#include "qoi.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void put_u32be(unsigned char *b, unsigned int v) {
    b[0] = (unsigned char)(v >> 24);
    b[1] = (unsigned char)(v >> 16);
    b[2] = (unsigned char)(v >> 8);
    b[3] = (unsigned char)v;
}

static int write_raw(const char *path, const void *pixels, unsigned w, unsigned h, int channels) {
    FILE *f = fopen(path, "wb");
    if (!f) return 0;
    unsigned char hdr[9];
    put_u32be(hdr, w);
    put_u32be(hdr + 4, h);
    hdr[8] = (unsigned char)channels;
    size_t px = (size_t)w * h * channels;
    int ok = fwrite(hdr, 1, sizeof(hdr), f) == sizeof(hdr) && fwrite(pixels, 1, px, f) == px;
    fclose(f);
    return ok;
}

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: mkraw png2raw <in.png> <out.raw>\n");
        fprintf(stderr, "       mkraw qoi2raw <in.qoi> <out.raw>\n");
        return 1;
    }
    if (strcmp(argv[1], "png2raw") == 0) {
        int w, h, n;
        if (!stbi_info(argv[2], &w, &h, &n)) {
            fprintf(stderr, "png2raw: can't read header %s\n", argv[2]);
            return 1;
        }
        if (n != 3) n = 4;
        unsigned char *p = (unsigned char *)stbi_load(argv[2], &w, &h, NULL, n);
        if (!p) {
            fprintf(stderr, "png2raw: load failed %s\n", argv[2]);
            return 1;
        }
        int ok = write_raw(argv[3], p, (unsigned)w, (unsigned)h, n);
        free(p);
        if (!ok) {
            fprintf(stderr, "png2raw: write failed %s\n", argv[3]);
            return 1;
        }
    }
    else if (strcmp(argv[1], "qoi2raw") == 0) {
        qoi_desc d;
        void *p = qoi_read(argv[2], &d, 0);
        if (!p) {
            fprintf(stderr, "qoi2raw: decode failed %s\n", argv[2]);
            return 1;
        }
        int ok = write_raw(argv[3], p, d.width, d.height, d.channels);
        free(p);
        if (!ok) {
            fprintf(stderr, "qoi2raw: write failed %s\n", argv[3]);
            return 1;
        }
    }
    else {
        fprintf(stderr, "mkraw: unknown mode %s\n", argv[1]);
        return 1;
    }
    return 0;
}
