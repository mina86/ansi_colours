/* ansi_colours – true-colour ↔ ANSI terminal palette converter
   Copyright 2018 by Michał Nazarewicz <mina86@mina86.com>

   ansi_colours is free software: you can redistribute it and/or modify it
   under the terms of the GNU Lesser General Public License as published by
   the Free Software Foundation; either version 3 of the License, or (at
   your option) any later version.

   ansi_colours is distributed in the hope that it will be useful, but
   WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser
   General Public License for more details.

   You should have received a copy of the GNU Lesser General Public License
   along with ansi_colours.  If not, see <http://www.gnu.org/licenses/>. */

#include <stdbool.h>
#include <stdint.h>

#include "ansi_colours.h"


static uint32_t cube_value(uint8_t r);
static uint32_t cube_index_red(uint8_t r);
static uint32_t cube_index_green(uint8_t g);
static uint32_t cube_index_blue(uint8_t b);
static uint8_t luminance(uint32_t rgb);
static uint32_t distance(uint32_t x, uint32_t y);


#define R(c) (((c) >> 16) & 0xff)
#define G(c) (((c) >>  8) & 0xff)
#define B(c) ( (c)        & 0xff)


/* Returns sRGB colour corresponding to the index in the 256-colour ANSI
   palette. */
uint32_t rgb_from_ansi256(uint8_t index) {
	/* The 16 system colours as used by default by xterm.  Taken from
	   XTerm-col.ad distributed with xterm source code. */
	static uint32_t system_colours[16] = {
		0x000000, 0xce0000, 0x00ce00, 0xcece00,
		0x0000ee, 0xce00ce, 0x00cece, 0xefefef,
		0x7f7f7f, 0xff0000, 0x00ff00, 0xffff00,
		0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,
	};

	if (!(index & 0xf0)) {  /* equivalent to index < 16 */
		return system_colours[index];
	} else if (index < 232) {
		index -= 16;
		return ((cube_value(index / 36    ) << 16) |
			(cube_value(index /  6 % 6) <<  8) |
			(cube_value(index      % 6)));
	} else {
		index = (index - 232) * 10 + 8;
		return (uint32_t)index * 0x010101;
	}
}


/* Returns index of a colour in 256-colour ANSI palette approximating given sRGB
   colour. */
uint8_t ansi256_from_rgb(uint32_t rgb) {
	/* A lookup table for approximations of shades of grey.  Values chosen
	   to get smallest possible ΔE*₀₀.  A lookup table is used because
	   calculating correct mapping has several corner cases.  For one, the
	   greyscale ramp starts at rgb(8, 8, 8) but ends at rgb(238, 238, 238)
	   resulting in asymmetric distance to the extreme values.  For another,
	   shades of grey are present in the greyscale ramp as well as the 6×6×6
	   colour cube making it necessary to consider multiple cases. */
	static const uint8_t ansi256_from_grey[256] = {
		 16,  16,  16,  16,  16, 232, 232, 232,
		232, 232, 232, 232, 232, 232, 233, 233,
		233, 233, 233, 233, 233, 233, 233, 233,
		234, 234, 234, 234, 234, 234, 234, 234,
		234, 234, 235, 235, 235, 235, 235, 235,
		235, 235, 235, 235, 236, 236, 236, 236,
		236, 236, 236, 236, 236, 236, 237, 237,
		237, 237, 237, 237, 237, 237, 237, 237,
		238, 238, 238, 238, 238, 238, 238, 238,
		238, 238, 239, 239, 239, 239, 239, 239,
		239, 239, 239, 239, 240, 240, 240, 240,
		240, 240, 240, 240,  59,  59,  59,  59,
		 59, 241, 241, 241, 241, 241, 241, 241,
		242, 242, 242, 242, 242, 242, 242, 242,
		242, 242, 243, 243, 243, 243, 243, 243,
		243, 243, 243, 244, 244, 244, 244, 244,
		244, 244, 244, 244, 102, 102, 102, 102,
		102, 245, 245, 245, 245, 245, 245, 246,
		246, 246, 246, 246, 246, 246, 246, 246,
		246, 247, 247, 247, 247, 247, 247, 247,
		247, 247, 247, 248, 248, 248, 248, 248,
		248, 248, 248, 248, 145, 145, 145, 145,
		145, 249, 249, 249, 249, 249, 249, 250,
		250, 250, 250, 250, 250, 250, 250, 250,
		250, 251, 251, 251, 251, 251, 251, 251,
		251, 251, 251, 252, 252, 252, 252, 252,
		252, 252, 252, 252, 188, 188, 188, 188,
		188, 253, 253, 253, 253, 253, 253, 254,
		254, 254, 254, 254, 254, 254, 254, 254,
		254, 255, 255, 255, 255, 255, 255, 255,
		255, 255, 255, 255, 255, 255, 255, 231,
		231, 231, 231, 231, 231, 231, 231, 231,
	};

	/* First of, if it’s shade of grey, we know exactly the best colour that
	   approximates it. */
	if (R(rgb) == G(rgb) && G(rgb) == B(rgb)) {
		return ansi256_from_grey[rgb & 0xff];
	}

	uint8_t grey_index = ansi256_from_grey[luminance(rgb)];
	uint32_t grey_distance = distance(rgb, rgb_from_ansi256(grey_index));
	uint32_t cube = cube_index_red(R(rgb)) + cube_index_green(G(rgb)) +
		cube_index_blue(B(rgb));
	return distance(rgb, cube) < grey_distance ? cube >> 24 : grey_index;
}


/* The next three functions approximate a pure colour by a colour in the 6×6×6
   colour cube.  E.g. cube_index_red(r) approximates an rgb(r, 0, 0) colour.
   This was motivated by ΔE*₀₀ being most variable in dark colours so I felt
   it’s more important to better approximate dark colours than white colours.

   The return values of the functions is kinda weird but it makes
   ansi256_from_rgb a bit shorter, as in having to do a bit fewer things. */

#define CUBE_THRESHOLDS(a, b, c, d, e)		\
	if      (v < a) return IDX(0,   0);	\
	else if (v < b) return IDX(1,  95);	\
	else if (v < c) return IDX(2, 135);	\
	else if (v < d) return IDX(3, 175);	\
	else if (v < e) return IDX(4, 215);	\
	else            return IDX(5, 255);

#define IDX(i, v) ((((uint32_t)i * 36 + 16) << 24) | ((uint32_t)v << 16))

static uint32_t cube_index_red(uint8_t v) {
	CUBE_THRESHOLDS(38, 115, 155, 196, 235);
}

#undef IDX
#define IDX(i, v) ((((uint32_t)i * 6) << 24) | ((uint32_t)v << 8))

static uint32_t cube_index_green(uint8_t v) {
	CUBE_THRESHOLDS(36, 116, 154, 195, 235);
}

#undef IDX
#define IDX(i, v) (((uint32_t)i << 24) | (uint32_t)v)

static uint32_t cube_index_blue(uint8_t v) {
	CUBE_THRESHOLDS(35, 115, 155, 195, 235);
}

#undef IDX
#undef CUBE_THRESHOLDS


/* Translates an index in the 6×6×6 colour cube to a value in sRGB space.  The
   argument must be less than six. */
static uint32_t cube_value(uint8_t idx) {
	static const uint8_t values[] = {0, 95, 135, 175, 215, 255};
	return values[idx];
}


/* Returns luminance of given sRGB colour.  The calculation favours speed over
   precision and so doesn’t correctly account for sRGB’s gamma correction. */
static uint8_t luminance(uint32_t rgb) {
	/* The following weighted average is as fast as naive arithmetic mean
	   and at the same time noticeably more prices.  The coefficients are
	   the second row of the RGB->XYZ conversion matrix (i.e. values for
	   calculating Y from linear RGB) which I’ve calculated so that
	   denominator is 2^²⁴ to simplify division. */
	return ((uint32_t) 3568058 * R(rgb) +
	        (uint32_t)11998262 * G(rgb) +
	        (uint32_t) 1210896 * B(rgb)) >> 24;

	/* Approximating sRGB gamma correction with a simple γ=2 improves the
	   precision considerably but is also five times slower than the above
	   (and probably slower still on architectures lacking MMS or FPU).

	       return sqrtf((float)r * (float)r * 0.2126729f +
	                    (float)g * (float)g * 0.7151521f +
	                    (float)b * (float)b * 0.0721750);

	   Doing proper gamma correction results in further improvement but is
	   also 20 times slower, so we’re opting out from doing that. */
}


/* Calculates distance between two colours.  Tries to balance speed and
   perceptual correctness.  It’s not a proper metric but two properties this
   function provides are: d(x, x) = 0 and d(x, y) < d(x, z) implies x being
   closer to y than to z. */
static uint32_t distance(uint32_t x, uint32_t y) {
	/* See <https://www.compuphase.com/cmetric.htm> though we’re doing a few
	   things to avoid some of the calculations.  We can do that since we
	   only care about some properties of the metric. */
	int32_t r_sum = R(x) + R(y);
	int32_t r = (int32_t)R(x) - (int32_t)R(y);
	int32_t g = (int32_t)G(x) - (int32_t)G(y);
	int32_t b = (int32_t)B(x) - (int32_t)B(y);
	return (1024 + r_sum) * r * r + 2048 * g * g + (1534 - r_sum) * b * b;
}
