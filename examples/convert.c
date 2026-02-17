/* Usage example for the ansi_colours library.  Build with `make`. */

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include <ansi_colours.h>


static uint32_t parse_u8(const char *argv0, const char *arg);


int main(int argc, char **argv) {
	switch (argc) {
	case 2: {
		uint32_t index = parse_u8(argv[0], argv[1]);
		if (index == (uint32_t)-1) {
			break;
		}
		uint32_t rgb = rgb_from_ansi256(index);
		printf("%3d: #%06x\n", index, rgb);
		return 0;
	}

	case 4: {
		uint32_t r, g, b;
		if ((r = parse_u8(argv[0], argv[1])) == (uint32_t)-1 ||
		    (g = parse_u8(argv[0], argv[2])) == (uint32_t)-1 ||
		    (b = parse_u8(argv[0], argv[3])) == (uint32_t)-1) {
			break;
		}
		uint32_t rgb = (r << 16) | (g << 8) | b;
		uint8_t index = ansi256_from_rgb(rgb);
		uint32_t approx = rgb_from_ansi256(index);
		printf("#%06x ~ %3d #%06x\n", rgb, index, approx);
		return 0;
	}

	default:
		fprintf(stderr, "usage: %s ( <index> | <r> <g> <b> )\n",
		        argv[0]);
	}

	return 1;
}


static uint32_t parse_u8(const char *argv0, const char *arg) {
	errno = 0;
	char *ptr;
	unsigned long value = strtoul(arg, &ptr, 10);
	if (value <= 255 && !errno && *ptr == 0) {
		return value;
	}
	fprintf(stderr, "%s: expected 8-bit unsigned integer: ‘%s’\n",
	        argv0, arg);
	return -1;
}
