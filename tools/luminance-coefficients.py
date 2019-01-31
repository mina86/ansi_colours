# -*- coding: utf-8 -*-

## Script for calculating RGB↔XYZ transformation matrices
## Copyright 2018-2019 by Michał Nazarewicz <mina86@mina86.com>
##
## This script is free software: you can redistribute it and/or modify it
## under the terms of the GNU Lesser General Public License as published by
## the Free Software Foundation; either version 3 of the License, or (at
## your option) any later version.
##
## This script is distributed in the hope that it will be useful, but
## WITHOUT ANY WARRANTY; without even the implied warranty of
## MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser
## General Public License for more details.
##
## You should have received a copy of the GNU Lesser General Public License
## along with this script.  If not, see <http://www.gnu.org/licenses/>.

from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

import collections
import fractions
import functools
import math
import sys


################################################################################
#### RGB→XYZ transformation matrix calculation logic
################################################################################

def inverse_3x3_matrix(matrix):
    """Returns inversion of 3✕3 matrix M, i.e. M⁻¹.

    The matrix argument specifies matrix to find inverse of represented as
    a sequence of three rows where each row is in turn a sequence of three
    values.

    Behaviour is undefined if inverse of the matrix cannot be calculated,
    i.e. if any of the rows in it is a linear combination of the remaining rows.
    """
    def cofactor(row, col):
        a, b, c, d = [matrix[r][c]
                      for r in (0, 1, 2) if r != row
                      for c in (0, 1, 2) if c != col]
        det_minor = a * d - b * c
        return det_minor if (row ^ col) & 1 == 0 else -det_minor

    comatrix = tuple(tuple(cofactor(row, col) for col in (0, 1, 2))
                     for row in (0, 1, 2))
    # https://en.wikipedia.org/wiki/Minor_(linear_algebra)#Cofactor_expansion_of_the_determinant
    det = sum(matrix[0][col] * comatrix[0][col] for col in (0, 1, 2))
    # https://en.wikipedia.org/wiki/Minor_(linear_algebra)#Inverse_of_a_matrix
    return tuple(tuple(comatrix[col][row] / det for col in (0, 1, 2))
                 for row in (0, 1, 2))


def mul_matrix_by_column(matrix, column):
    """Multiplies m✕n matrix by n✕1 column.

    The matrix argument specifies matrix to multiply, i.e. the left operand of
    the multiplication, represented as a sequence of rows where each row is in
    turn a sequence of values.  Each row must have the same number of elements
    and must equal number of elements in column argument.

    The column argument is a sequence of values and is interpreted as an n✕1
    matrix.  It’s the right operand of the multiplication operator.

    The function returns result of the multiplication, i.e. an m✕1 matrix result
    of the ‘matrix ✕ column’ operation.
    """
    return tuple(sum(row[i] * column[i] for i in range(len(row)))
                 for row in matrix)

def mul_matrix_by_diag(matrix, column):
    """Multiplies an m✕n matrix by an n✕n diagonal matrix.

    The matrix argument specifies matrix to multiply, i.e. the left operand of
    the multiplication, represented as a sequence of rows where each row is in
    turn a sequence of values.  Each row must have the same number of elements
    and must equal number of elements in column argument.

    The column argument is a sequence of values and is interpreted as an
    n✕1 matrix.  It’s used to construct an n✕n diagonal matrix [a_{i,j}] such
    that a_{i,j} = column[i] if i = j else 0.  This diagonal matrix is the right
    operand of the multiplication operator.

    The function returns result of the multiplication, i.e. an m✕n matrix result
    of the ‘matrix ✕ diag(column)’ operation.
    """
    return tuple(tuple(row[c] * column[c] for c in range(len(column)))
                 for row in matrix)


Chromaticity = collections.namedtuple('Chromaticity', 'x y')

def calculate_rgb_matrix(primaries, white):
    """Returns an RGB→XYZ transformation matrix for given primaries and white.

    The primaries argument is a sequence of three Chromaticity objects defining
    the primary colours of the RGB colour space.  In other words, a sequence of
    red, green and blue chromaticities given as x and y coordinates of the xyY
    space.

    The white argument is a Chromaticity object defining the reference white
    point, i.e. the x and y coordinates of the white point.  Y component is
    assumed to be one.

    The function returns a 3✕3 RGB→XYZ transformation matrix (represented as
    a three-element sequence of three-element sequences).
    """
    # Calculate the transformation matrix as per
    # https://mina86.com/2019/srgb-xyz-matrix/
    M_prime = (tuple(c.x / c.y             for c in primaries),
               tuple(1                     for _ in primaries),
               tuple((1 - c.x - c.y) / c.y for c in primaries))

    # Y = M′⁻¹ ✕ W
    W = (white.x / white.y, 1, (1 - white.x - white.y) / white.y)
    Y = mul_matrix_by_column(inverse_3x3_matrix(M_prime), W)

    # M = M′ ✕ diag(Y)
    return mul_matrix_by_diag(M_prime, Y)


################################################################################
#### sRGB specific stuff
################################################################################

def calculate_srgb_matrix():
    """Returns an sRGB→XYZ transformation matrix."""
    # https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.709-6-201506-I!!PDF-E.pdf
    primaries = (
        Chromaticity(fractions.Fraction(64, 100), fractions.Fraction(33, 100)),
        Chromaticity(fractions.Fraction(30, 100), fractions.Fraction(60, 100)),
        Chromaticity(fractions.Fraction(15, 100), fractions.Fraction( 6, 100))
    )
    white = Chromaticity(fractions.Fraction(31271, 100000),
                         fractions.Fraction(32902, 100000))

    return calculate_rgb_matrix(primaries, white)


################################################################################
#### Pretty printing code
################################################################################

def _print_matrix(name, eq_sign, fmt, formatted):
    rows = len(formatted)

    def char(row, chrs):
        if rows == 1:
            return chrs[0]
        elif row == 0:
            return chrs[1]
        elif row < rows - 1:
            return chrs[2]
        else:
            return chrs[3]

    prefix = '{} {}'.format(name, eq_sign)
    prefix_at = int(rows / 2)
    padding = ' ' * len(prefix)
    for row in range(len(formatted)):
        print(prefix if row == prefix_at else padding,
              char(row, '[⎡⎢⎣'), fmt.format(*formatted[row]), char(row, ']⎤⎥⎦'))


def print_matrix_real(name, matrix):
    rows = len(matrix)
    cols = len(matrix[0])

    # Format all the values as real numbers.  To make alignment work nicely, if
    # at least one real number in the column is negative, prefix all
    # non-negative by a space.
    has_negative = [any(matrix[r][c] < 0 for r in range(rows))
                    for c in range(cols)]
    formatted = []
    for row in matrix:
        fmt_row = []
        for c in range(cols):
            v = row[c]
            f = ' {!r}' if v >= 0 and has_negative[c] else '{!r}'
            fmt_row.append(f.format(v.numerator / v.denominator))
        formatted.append(fmt_row)

    widths = [max(len(formatted[r][c]) for r in range(rows))
              for c in range(len(formatted[0]))]
    fmt = ' '.join('{:%d}' % widths[c] for c in range(cols))

    _print_matrix(name, '≈', fmt, formatted)


def print_matrix_rational(name, matrix):
    gcd = getattr(math, 'gcd', fractions.gcd)  # Python2 compatibility
    rows = len(matrix)

    # Format all the values as rational numbers, i.e. each number results in two
    # formatted integers.  To make comparison easier, values in each row are
    # brought to the same denominator.
    formatted = []
    for row in matrix:
        fmt_row = []
        common = functools.reduce(lambda x, y: x * y // gcd(x, y),
                                  (v.denominator for v in row))
        for v in row:
            fmt_row.append(str(v.numerator * common // v.denominator))
            fmt_row.append(str(common))
        formatted.append(fmt_row)

    widths = [max(len(formatted[r][c]) for r in range(rows))
              for c in range(len(formatted[0]))]
    fmt = '  '.join('{:>%d} / {:%d}' % (widths[c], widths[c + 1])
                    for c in range(0, len(widths), 2))

    _print_matrix(name, '=', fmt, formatted)


def print_matrix(name, matrix):
    print_matrix_real(name, matrix)
    print_matrix_rational(name, matrix)
    print()



################################################################################
#### main
################################################################################

def main():
    if sys.version_info[0] == 2:
        import codecs
        sys.stdout = codecs.getwriter('utf8')(sys.stdout)

    M = calculate_srgb_matrix()

    print_matrix('M', M)
    print_matrix('M⁻¹', inverse_3x3_matrix(M))

    Y = M[1]
    vec = [repr(v.numerator / v.denominator) for v in Y]
    print('Y ≈ [{}]'.format(' '.join(vec)))
    for bits in (56, 24, 16, 8):
        d = 1 << bits
        vec = [int(v.numerator * d // v.denominator) for v in Y]
        got_d = sum(vec)
        while got_d < d:
            idx = max(range(len(vec)),
                      key=lambda i: Y[i] - fractions.Fraction(vec[i], got_d))
            vec[idx] += 1
            got_d += 1
        print('Y ≈ {} / 2**{}'.format(vec, bits))


if __name__ == '__main__':
    main()
