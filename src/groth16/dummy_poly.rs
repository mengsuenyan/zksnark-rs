use super::*;

#[derive(Clone, PartialEq, Debug)]
pub struct DummyPoly {
    coeffs: Vec<Z251>,
}

impl DummyPoly {
    #[cfg(test)]
    fn is_zero(&self) -> bool {
        for &c in self.coeffs.as_slice() {
            if c != Z251::add_identity() {
                return false;
            }
        }

        true
    }
}

impl From<Vec<Z251>> for DummyPoly {
    fn from(coeffs: Vec<Z251>) -> Self {
        DummyPoly { coeffs }
    }
}

impl Polynomial<Z251> for DummyPoly {
    fn coefficients(&self) -> Vec<Z251> {
        self.coeffs.clone()
    }
}

impl Add for DummyPoly {
    type Output = DummyPoly;

    fn add(self, rhs: Self) -> Self::Output {
        let coeffs = if self.coeffs.len() < rhs.coeffs.len() {
            self.coeffs
                .into_iter()
                .chain(repeat(0.into()))
                .zip(rhs.coeffs.into_iter())
                .map(|(a, b)| a + b)
                .collect::<Vec<_>>()
        } else {
            rhs.coeffs
                .into_iter()
                .chain(repeat(0.into()))
                .zip(self.coeffs.into_iter())
                .map(|(a, b)| a + b)
                .collect::<Vec<_>>()
        };

        DummyPoly { coeffs }
    }
}

impl Neg for DummyPoly {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.coeffs.as_mut_slice().iter_mut().for_each(|a| *a = -*a);

        self
    }
}

impl Sub for DummyPoly {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Sum for DummyPoly {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(
            DummyPoly {
                coeffs: vec![Z251::from(0)],
            },
            |acc, x| acc + x,
        )
    }
}

impl Mul for DummyPoly {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self::Output {
        use std::cmp::max;

        self.remove_leading_zeros();
        rhs.remove_leading_zeros();

        let d = self.degree() + rhs.degree() + 1;
        let max_deg = max(self.degree(), rhs.degree());
        let coeffs = (0..d)
            .map(|i| {
                let self_iter = self.coeffs
                    .as_slice()
                    .iter()
                    .rev()
                    .take(self.degree() + max_deg + 1 - i)
                    .skip(max(self.degree() as isize - i as isize, 0) as usize);
                let rhs_iter = rhs.coeffs.as_slice().iter().take(i + 1).skip(max(
                    i as isize - self.degree() as isize,
                    0,
                )
                    as usize);

                self_iter
                    .zip(rhs_iter)
                    .fold(Z251::from(0), |acc, (&a, &b)| acc + a * b)
            })
            .collect::<Vec<_>>();

        DummyPoly { coeffs }
    }
}

impl Mul<Z251> for DummyPoly {
    type Output = Self;

    fn mul(mut self, rhs: Z251) -> Self::Output {
        self.coeffs
            .as_mut_slice()
            .iter_mut()
            .for_each(|c| *c = *c * rhs);

        self
    }
}

impl Div for DummyPoly {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        polynomial_division(self, rhs).0
    }
}

impl<I, J> From<(I, J)> for DummyPoly
where
    I: Iterator<Item = Z251>,
    J: Iterator<Item = (Z251, Z251)>,
{
    fn from((roots, points): (I, J)) -> Self {
        let roots_vec = roots.collect::<Vec<_>>();
        points
            .map(|(x, y)| lagrange_basis(roots_vec.clone().into_iter(), x) * y)
            .sum()
    }
}

fn lagrange_basis<I>(roots: I, x: Z251) -> DummyPoly
where
    I: Iterator<Item = Z251>,
{
    roots
        .filter(|&r| r != x)
        .fold(vec![1.into()].into(), |acc, m| {
            DummyPoly::from(vec![-m, 1.into()]) * (Z251::from(1) / (x - m)) * acc
        })
}

pub fn root_poly<I>(roots: I) -> DummyPoly
where
    I: Iterator<Item = Z251>,
{
    roots.fold(DummyPoly::from(vec![1.into()]), |acc, r| {
        acc * (DummyPoly::from(vec![-r, 1.into()]))
    })
}

#[test]
fn dummy_add() {
    // Trivial addition
    let a = DummyPoly::from(vec![]);
    let b = DummyPoly::from(vec![]);
    assert!((a + b).is_zero());

    // Addition with one trivial term
    let a = DummyPoly::from(vec![]);
    let b = vec![Z251::from(1), Z251::from(2), Z251::from(3)].into();
    assert_eq!(
        a + b,
        vec![Z251::from(1), Z251::from(2), Z251::from(3)].into()
    );

    // Addition with one zero term
    let a = DummyPoly::from(vec![Z251::from(0)]);
    let b = vec![Z251::from(1), Z251::from(2), Z251::from(3)].into();
    assert_eq!(
        a + b,
        vec![Z251::from(1), Z251::from(2), Z251::from(3)].into()
    );

    // Addition with leading zeros
    let a = DummyPoly::from(vec![Z251::from(4), Z251::from(5), Z251::from(6)]);
    let b = vec![Z251::from(1), Z251::from(2), Z251::from(3), Z251::from(0)].into();
    assert_eq!(
        a + b,
        vec![Z251::from(5), Z251::from(7), Z251::from(9), Z251::from(0)].into()
    );

    // Addition with overflow
    let a = DummyPoly::from(vec![Z251::from(234), Z251::from(100), Z251::from(6)]);
    let b = vec![Z251::from(123), Z251::from(234), Z251::from(3)].into();
    assert_eq!(
        a + b,
        vec![Z251::from(106), Z251::from(83), Z251::from(9)].into()
    );
}

#[test]
fn dummy_neg() {
    // Check to see if the negative is the additive inverse
    // Generate random quadratic polynomials

    for _ in 0..1000 {
        let a = DummyPoly::from(vec![
            Z251::random_elem(),
            Z251::random_elem(),
            Z251::random_elem(),
        ]);
        let b = -a.clone();
        assert!((a + b).is_zero());
    }
}

#[test]
fn dummy_sub() {
    // Check that if c = a - b then a = b + c
    for _ in 0..1000 {
        let a = DummyPoly::from(vec![
            Z251::random_elem(),
            Z251::random_elem(),
            Z251::random_elem(),
        ]);
        let b = DummyPoly::from(vec![
            Z251::random_elem(),
            Z251::random_elem(),
            Z251::random_elem(),
        ]);
        let c = a.clone() - b.clone();
        assert_eq!(a, b + c);
    }
}

#[test]
fn dummy_sum() {
    let mut polys = Vec::with_capacity(20);
    let mut sum: DummyPoly;

    for _ in 0..1000 {
        polys.clear();
        sum = vec![Z251::add_identity(); 3].into();

        for _ in 0..20 {
            let a = DummyPoly::from(vec![
                Z251::random_elem(),
                Z251::random_elem(),
                Z251::random_elem(),
            ]);
            polys.push(a.clone());
            sum = sum + a;
        }

        assert_eq!(sum, polys.clone().into_iter().sum());
    }
}

#[test]
fn dummy_mul() {
    // Trivial multiplication
    let a = DummyPoly::from(vec![]);
    let b = DummyPoly::from(vec![]);
    assert!((a * b).is_zero());

    // Multiplication with one trivial term
    let a = DummyPoly::from(vec![]);
    let b = DummyPoly::from(vec![Z251::from(1), Z251::from(2), Z251::from(3)]);
    assert!((a * b).is_zero());

    // Multiplication with one zero term
    let a = DummyPoly::from(vec![Z251::from(0)]);
    let b = DummyPoly::from(vec![Z251::from(1), Z251::from(2), Z251::from(3)]);
    assert!((a * b).is_zero());

    // Multiplication with leading zeros
    let a = DummyPoly::from(vec![Z251::from(4), Z251::from(5), Z251::from(6)]);
    let b = DummyPoly::from(vec![
        Z251::from(1),
        Z251::from(2),
        Z251::from(3),
        Z251::from(0),
    ]);
    assert_eq!(
        a * b,
        vec![
            Z251::from(4),
            Z251::from(13),
            Z251::from(28),
            Z251::from(27),
            Z251::from(18),
        ].into()
    );

    // Multiplication with overflow
    let a = DummyPoly::from(vec![Z251::from(234), Z251::from(100), Z251::from(6)]);
    let b = DummyPoly::from(vec![Z251::from(123), Z251::from(234), Z251::from(3)]);
    assert_eq!(
        a * b,
        vec![
            Z251::from(168),
            Z251::from(39),
            Z251::from(242),
            Z251::from(198),
            Z251::from(18),
        ].into()
    );
}

#[test]
fn dummy_scalar_mul() {
    // Scalar multiplication with trivial polynomial
    let a = DummyPoly::from(vec![]);
    let s = Z251::from(69);
    assert!((a * s).is_zero());

    // Scalar multiplication with zero polynomial
    let a = DummyPoly::from(vec![0.into()]);
    let s = Z251::from(69);
    assert!((a * s).is_zero());

    // Scalar multiplication with non-zero polynomial
    let a = DummyPoly::from(vec![Z251::from(1), Z251::from(2), Z251::from(3)]);
    let s = Z251::from(69);
    assert_eq!(
        a * s,
        DummyPoly::from(vec![Z251::from(69), Z251::from(138), Z251::from(207)])
    );

    // Scalar multiplication with overflow
    let a = DummyPoly::from(vec![Z251::from(20), Z251::from(2), Z251::from(3)]);
    let s = Z251::from(69);
    assert_eq!(
        a * s,
        DummyPoly::from(vec![Z251::from(125), Z251::from(138), Z251::from(207)])
    );

    // Scalar multiplication zero scalar
    let a = DummyPoly::from(vec![Z251::from(20), Z251::from(2), Z251::from(3)]);
    let s = Z251::from(0);
    assert!((a * s).is_zero());
}

#[test]
fn dummy_div() {
    // Check that if a * b = c then a = c / b

    for _ in 0..1000 {
        let mut a = DummyPoly::from(vec![
            Z251::random_elem(),
            Z251::random_elem(),
            Z251::random_elem(),
        ]);
        let b = DummyPoly::from(vec![
            Z251::random_elem(),
            Z251::random_elem(),
            Z251::random_elem(),
        ]);
        if b.is_zero() {
            continue;
        }
        a.remove_leading_zeros();
        let c = a.clone() * b.clone();

        assert_eq!(a, c / b);
    }
}

#[test]
fn dummy_lagrange() {
    for max in 2..25 {
        for i in 1..max {
            let roots = (1..max).map(|x| x.into());
            let poly = lagrange_basis(roots, i.into());

            for j in 1..max {
                if i == j {
                    assert_eq!(poly.evaluate(j.into()), Z251::mul_identity());
                } else {
                    assert_eq!(poly.evaluate(j.into()), Z251::add_identity());
                }
            }
        }
    }
}

#[test]
fn dummy_from_roots() {
    for mask in 1..255_u8 {
        let roots = (1..9).map(|x| x.into());
        let points = (0..8_u8)
            .filter(|i| (1_u8 << i) & mask != 0)
            .map(|i| ((i as usize + 1).into(), (i as usize + 2).into()));
        let poly = DummyPoly::from((roots, points));

        for i in 0..8_u8 {
            if (1_u8 << i) & mask != 0 {
                assert_eq!(
                    poly.evaluate((i as usize + 1).into()),
                    (i as usize + 2).into()
                );
            } else {
                assert_eq!(poly.evaluate((i as usize + 1).into()), Z251::add_identity());
            }
        }
    }
}

#[test]
fn dummy_root_poly() {
    for i in 2..25 {
        let poly = root_poly((1..i).map(|x| x.into()));

        for j in 1..i {
            assert_eq!(poly.evaluate(j.into()), Z251::add_identity());
        }
    }
}
