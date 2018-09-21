import collections
import fractions


def inverse(M):
    def signed_minor_det(row, col):
        a, b, c, d = [M[r][c]
                      for r in (0, 1, 2) if r != row
                      for c in (0, 1, 2) if c != col]
        res = a * d - b * c
        return res if (row ^ col) & 1 == 0 else -res

    signed_minors = [
        [signed_minor_det(row, col) for col in (0, 1, 2)] for row in (0, 1, 2)
    ]
    det = sum(M[0][col] * signed_minors[0][col] for col in (0, 1, 2))
    return [[signed_minors[col][row] / det for col in (0, 1, 2)]
            for row in (0, 1, 2)]


def str_fractions(vv):
    common = vv[0].denominator
    for v in vv:
        common = int(common * v.denominator / fractions.gcd(common, v.denominator))
    return ['{} / {}'.format(int(v.numerator * common // v.denominator), common)
            for v in vv]


def main():
    xy = collections.namedtuple('xy', 'x y')

    # https://en.wikipedia.org/wiki/SRGB#The_sRGB_gamut
    r = xy(fractions.Fraction(64, 100), fractions.Fraction(33, 100))
    g = xy(fractions.Fraction(30, 100), fractions.Fraction(60, 100))
    b = xy(fractions.Fraction(15, 100), fractions.Fraction( 6, 100))

    # https://en.wikipedia.org/wiki/Illuminant_D65#Definition
    W = [
        fractions.Fraction(95047, 100000), 1, fractions.Fraction(108883, 100000)
    ]

    # Calculating XYZ for D65 from x and y values gives different result than
    # the above and resulting matrix is further away from RGB->XYZ
    # transformation matrices published on the Internet so let's stick to the
    # values above.
    #
    # xw = fractions.Fraction(31271, 10000)
    # yw = fractions.Fraction(32902, 10000)
    # W = [xw / yw, 1, (1 - xw - yw) / yw]

    # Calculate the transformation matrix as per
    # http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html
    matrix = [[c.x / c.y for c in (r, g, b)],
              [1, 1, 1],
              [(1 - c.x - c.y) / c.y for c in (r, g, b)]]
    inv = inverse(matrix)
    S = tuple(sum(W[c] * inv[r][c] for c in (0, 1, 2)) for r in (0, 1, 2))
    M = [[matrix[r][c] * S[c] for c in (0, 1, 2)] for r in (0, 1, 2)]

    # Print M and inverted M
    print('[M] =')
    for row in M:
        print('  {:-20} {:-20} {:-20}   {} {} {}'.format(
            *[v.numerator / v.denominator for v in row],
            *str_fractions(row),
        ))
    print()

    print('[M]^-1 =')
    for row in inverse(M):
        print('  {:-20} {:-20} {:-20}   {} {} {}'.format(
            *[v.numerator / v.denominator for v in row],
            *[str(v) for v in row],
        ))
    print()

    # Figure out best luminance calculations.
    denoms = [v.denominator for v in M[1]]
    denom = denoms[0] * denoms[1] / fractions.gcd(denoms[0], denoms[1])
    denom = denom * denoms[2] / fractions.gcd(denom, denoms[2])
    for d in (1 << 56, 1 << 24, 1 << 16):
        vec = [round(v.numerator * d / v.denominator) for v in M[1]]
        print(vec, sum(vec), d)


if __name__ == '__main__':
    main()
