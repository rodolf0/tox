// Ref: https://en.wikipedia.org/wiki/International_System_of_Units
// Quantity: the whole thing
// Value: the number
// Prefix: mega, kilo, etc.
// Unit: second, metre, etc

use std::cmp::Ordering;
use std::fmt;
use std::ops;

const fn magnitude_prefix(factor: i32) -> Option<(&'static str, &'static str)> {
    Some(match factor {
        -24 => ("y", "yocto"),
        -21 => ("z", "zepto"),
        -18 => ("a", "atto"),
        -15 => ("f", "femto"),
        -12 => ("p", "pico"),
        -9 => ("n", "nano"),
        -6 => ("µ", "micro"),
        -3 => ("m", "milli"),
        -2 => ("c", "centi"),
        -1 => ("d", "deci"),
        0 => ("", ""),
        1 => ("da", "deca"),
        2 => ("h", "hecto"),
        3 => ("k", "kilo"),
        6 => ("M", "mega"),
        9 => ("G", "giga"),
        12 => ("T", "tera"),
        15 => ("P", "peta"),
        18 => ("E", "exa"),
        21 => ("Z", "zetta"),
        24 => ("Y", "yotta"),
        _ => return None,
    })
}

// Reduce mantisa to complement with magnitude prefix
fn normalize(value: f64) -> (f64, i32) {
    let factor = value.abs().log10() as i32;
    // Round factor to multiple of 3.
    // For abs values < 1.0 decrease the value for > 1.0 mantisa.
    // Clamp factor to 24, prefixes for which we have names.
    let factor = if value.abs() < 1.0 {
        (-3 + factor - factor % 3).max(-24)
    } else {
        (factor - factor % 3).min(24)
    };
    (value / 10.0_f64.powi(factor), factor)
}

/// The physical dimension of a quantity, expressed as powers of the seven SI base units.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Dimension {
    s: i8,
    m: i8,
    kg: i8,
    A: i8,
    K: i8,
    mol: i8,
    cd: i8,
}

impl Dimension {
    #[rustfmt::skip]
    const fn names(&self) -> Option<(&'static str, &'static str)> {
        Some(match self {
            // Base
            Dimension {s: 1, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 0} => ("s", "second"),
            Dimension {s: 0, m: 1, kg: 0, A: 0, K: 0, mol: 0, cd: 0} => ("m", "meter"),
            Dimension {s: 0, m: 0, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("kg", "kilogram"),
            Dimension {s: 0, m: 0, kg: 0, A: 1, K: 0, mol: 0, cd: 0} => ("A", "ampere"),
            Dimension {s: 0, m: 0, kg: 0, A: 0, K: 1, mol: 0, cd: 0} => ("K", "kelvin"),
            Dimension {s: 0, m: 0, kg: 0, A: 0, K: 0, mol: 1, cd: 0} => ("mol", "mole"),
            Dimension {s: 0, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 1} => ("cd", "candela"),
            // Derived
            Dimension {s: -1, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 0} => ("Hz", "hertz"),
            Dimension {s: -2, m: 1, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("N", "newton"),
            Dimension {s: -2, m: -1, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("Pa", "pascal"),
            Dimension {s: -2, m: 2, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("J", "joule"),
            Dimension {s: -3, m: 2, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("W", "watt"),
            Dimension {s: 1, m: 0, kg: 0, A: 1, K: 0, mol: 0, cd: 0} => ("C", "coulomb"),
            Dimension {s: -3, m: 2, kg: 1, A: -1, K: 0, mol: 0, cd: 0} => ("V", "volt"),
            Dimension {s: 4, m: -2, kg: -1, A: 2, K: 0, mol: 0, cd: 0} => ("F", "farad"),
            Dimension {s: -3, m: 2, kg: 1, A: -2, K: 0, mol: 0, cd: 0} => ("Ω", "ohm"),
            Dimension {s: 3, m: -2, kg: -1, A: 2, K: 0, mol: 0, cd: 0} => ("S", "siemens"),
            Dimension {s: -2, m: 2, kg: 1, A: -1, K: 0, mol: 0, cd: 0} => ("Wb", "weber"),
            Dimension {s: -2, m: 0, kg: 1, A: -1, K: 0, mol: 0, cd: 0} => ("T", "tesla"),
            Dimension {s: -2, m: 2, kg: 1, A: -2, K: 0, mol: 0, cd: 0} => ("H", "henry"),
            Dimension {s: -1, m: 0, kg: 0, A: 0, K: 0, mol: 1, cd: 0} => ("kat", "katal"),
            _ => return None,
        })
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        static SUP: &[&str] = &[
            "\u{2070}", "\u{00b9}", "\u{00b2}", "\u{00b3}", "\u{2074}", "\u{2075}", "\u{2076}",
            "\u{2077}", "\u{2078}", "\u{2079}",
        ];
        macro_rules! fmtunit {
            ($dimension:ident, $dim_name:literal) => {
                match self.$dimension {
                    0 => None,
                    1 => Some($dim_name.to_string()),
                    n => Some(if n > 1 {
                        format!("{}{}", $dim_name, SUP[n as usize])
                    } else {
                        format!("{}\u{207b}{}", $dim_name, SUP[-n as usize])
                    }),
                }
            };
        }
        let mut dims = vec![
            (self.s, fmtunit!(s, "s")),
            (self.m, fmtunit!(m, "m")),
            (self.kg, fmtunit!(kg, "kg")),
            (self.A, fmtunit!(A, "A")),
            (self.K, fmtunit!(K, "K")),
            (self.mol, fmtunit!(mol, "mol")),
            (self.cd, fmtunit!(cd, "cd")),
        ];
        dims.sort();
        dims.reverse();
        write!(
            f,
            "{}",
            dims.into_iter()
                .filter_map(|x| x.1)
                .collect::<Vec<_>>()
                .join("\u{00b7}")
        )
    }
}

#[rustfmt::skip]
const UNITD: Dimension = Dimension {
    s: 0, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 0
};

impl ops::Mul for Dimension {
    type Output = Dimension;
    fn mul(self, rhs: Dimension) -> Dimension {
        Dimension {
            s: self.s + rhs.s,
            m: self.m + rhs.m,
            kg: self.kg + rhs.kg,
            A: self.A + rhs.A,
            K: self.K + rhs.K,
            mol: self.mol + rhs.mol,
            cd: self.cd + rhs.cd,
        }
    }
}

impl ops::Div for Dimension {
    type Output = Dimension;
    fn div(self, rhs: Dimension) -> Dimension {
        Dimension {
            s: self.s - rhs.s,
            m: self.m - rhs.m,
            kg: self.kg - rhs.kg,
            A: self.A - rhs.A,
            K: self.K - rhs.K,
            mol: self.mol - rhs.mol,
            cd: self.cd - rhs.cd,
        }
    }
}

/// A physical quantity consisting of a numeric value and a dimension.
#[derive(Clone, Copy, Debug)]
pub struct Quantity {
    value: f64,
    dimension: Dimension,
    display_name: Option<(&'static str, &'static str)>,
}

impl PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        self.dimension == other.dimension && self.value == other.value
    }
}

impl PartialOrd for Quantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.dimension == other.dimension {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

impl ops::Neg for Quantity {
    type Output = Quantity;
    fn neg(self) -> Self::Output {
        Quantity {
            value: -self.value,
            dimension: self.dimension,
            display_name: self.display_name,
        }
    }
}

impl ops::Mul<f64> for Quantity {
    type Output = Quantity;
    fn mul(self, rhs: f64) -> Self::Output {
        Quantity {
            value: self.value * rhs,
            dimension: self.dimension,
            display_name: self.display_name,
        }
    }
}

impl ops::Div<f64> for Quantity {
    type Output = Quantity;
    fn div(self, rhs: f64) -> Self::Output {
        Quantity {
            value: self.value / rhs,
            dimension: self.dimension,
            display_name: self.display_name,
        }
    }
}

impl ops::Mul<Quantity> for f64 {
    type Output = Quantity;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        Quantity {
            value: self * rhs.value,
            dimension: rhs.dimension,
            display_name: rhs.display_name,
        }
    }
}

impl ops::Div<Quantity> for f64 {
    type Output = Quantity;
    fn div(self, rhs: Self::Output) -> Self::Output {
        Quantity {
            value: self / rhs.value,
            dimension: UNITD / rhs.dimension,
            display_name: None,
        }
    }
}

impl ops::Mul<Quantity> for Quantity {
    type Output = Quantity;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        Quantity {
            value: self.value * rhs.value,
            dimension: self.dimension * rhs.dimension,
            display_name: None,
        }
    }
}

impl ops::Div<Quantity> for Quantity {
    type Output = Quantity;
    fn div(self, rhs: Self::Output) -> Self::Output {
        Quantity {
            value: self.value / rhs.value,
            dimension: self.dimension / rhs.dimension,
            display_name: None,
        }
    }
}

impl ops::Add<Quantity> for Quantity {
    type Output = Quantity;
    fn add(self, rhs: Self::Output) -> Self::Output {
        assert_eq!(self.dimension, rhs.dimension);
        Quantity {
            value: self.value + rhs.value,
            dimension: self.dimension,
            display_name: if self.display_name == rhs.display_name {
                self.display_name
            } else {
                None
            },
        }
    }
}

impl ops::Sub<Quantity> for Quantity {
    type Output = Quantity;
    fn sub(self, rhs: Self::Output) -> Self::Output {
        assert_eq!(self.dimension, rhs.dimension);
        Quantity {
            value: self.value - rhs.value,
            dimension: self.dimension,
            display_name: if self.display_name == rhs.display_name {
                self.display_name
            } else {
                None
            },
        }
    }
}

impl Quantity {
    const fn unit(
        dimension: Dimension,
        display_name: Option<(&'static str, &'static str)>,
    ) -> Quantity {
        Quantity {
            value: 1.0,
            dimension,
            display_name,
        }
    }

    /// The numeric value of this quantity in SI base units.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The dimension of this quantity.
    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// Whether this quantity has the same dimension as another.
    pub fn is_compatible_with(&self, other: &Quantity) -> bool {
        self.dimension == other.dimension
    }

    /// The short symbol for this quantity's unit (e.g. "m", "kg", "Ω").
    pub fn symbol(&self) -> String {
        self.display_name
            .map(|x| x.0.to_string())
            .or_else(|| self.dimension.names().map(|x| x.0.to_string()))
            .unwrap_or_else(|| self.dimension.to_string())
    }

    /// The full name for this quantity's unit (e.g. "meter", "kilogram").
    pub fn name(&self) -> Option<String> {
        self.display_name
            .map(|x| x.1.to_string())
            .or_else(|| self.dimension.names().map(|x| x.1.to_string()))
    }

    /// Absolute value.
    pub fn abs(&self) -> Quantity {
        Quantity {
            value: self.value.abs(),
            dimension: self.dimension,
            display_name: self.display_name,
        }
    }

    /// Raise to an integer power. Dimension exponents are multiplied by `n`.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if any dimension exponent overflows `i8`.
    pub fn powi(&self, n: i32) -> Quantity {
        Quantity {
            value: self.value.powi(n),
            dimension: Dimension {
                s: self.dimension.s * n as i8,
                m: self.dimension.m * n as i8,
                kg: self.dimension.kg * n as i8,
                A: self.dimension.A * n as i8,
                K: self.dimension.K * n as i8,
                mol: self.dimension.mol * n as i8,
                cd: self.dimension.cd * n as i8,
            },
            display_name: None,
        }
    }

    /// Square root. All dimension exponents must be even.
    ///
    /// # Panics
    ///
    /// Panics if any dimension exponent is not divisible by 2.
    pub fn sqrt(&self) -> Quantity {
        assert!(self.dimension.s % 2 == 0);
        assert!(self.dimension.m % 2 == 0);
        assert!(self.dimension.kg % 2 == 0);
        assert!(self.dimension.A % 2 == 0);
        assert!(self.dimension.K % 2 == 0);
        assert!(self.dimension.mol % 2 == 0);
        assert!(self.dimension.cd % 2 == 0);
        Quantity {
            value: self.value.sqrt(),
            dimension: Dimension {
                s: self.dimension.s / 2,
                m: self.dimension.m / 2,
                kg: self.dimension.kg / 2,
                A: self.dimension.A / 2,
                K: self.dimension.K / 2,
                mol: self.dimension.mol / 2,
                cd: self.dimension.cd / 2,
            },
            display_name: None,
        }
    }

    /// Convert this quantity to a given unit, returning the numeric value.
    ///
    /// # Panics
    ///
    /// Panics if `unit` has a different dimension.
    ///
    /// # Example
    ///
    /// ```
    /// use unidades::units::*;
    /// let dist = 500.0 * m;
    /// let km = 1000.0 * m;
    /// assert_eq!(dist.in_unit(km), 0.5);
    /// ```
    pub fn in_unit(&self, unit: Quantity) -> f64 {
        assert_eq!(self.dimension, unit.dimension);
        self.value / unit.value
    }
}

/// Helper to format a floating-point value respecting the formatter's width and precision.
fn write_value(f: &mut fmt::Formatter, value: f64) -> fmt::Result {
    if let Some(width) = f.width() {
        if let Some(prec) = f.precision() {
            write!(f, "{:width$.prec$}", value, width = width, prec = prec)
        } else {
            write!(f, "{:width$}", value, width = width)
        }
    } else if let Some(prec) = f.precision() {
        write!(f, "{:.prec$}", value, prec = prec)
    } else {
        write!(f, "{}", value)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name().is_some() {
            if self.dimension == units::kg.dimension {
                let (value, factor) = normalize(self.value * 1000.0);
                write_value(f, value)?;
                write!(f, " {}g", magnitude_prefix(factor).unwrap().0)
            } else {
                let (value, factor) = normalize(self.value);
                write_value(f, value)?;
                write!(
                    f,
                    " {}{}",
                    magnitude_prefix(factor).unwrap().0,
                    self.symbol()
                )
            }
        } else if self.dimension != UNITD {
            write_value(f, self.value)?;
            write!(f, " {}", self.symbol())
        } else {
            // Dimensionless, just add magnitude prefix
            let (value, factor) = normalize(self.value);
            write_value(f, value)?;
            if factor != 0 {
                write!(f, " {}", magnitude_prefix(factor).unwrap().0)
            } else {
                Ok(())
            }
        }
    }
}

/// Predefined SI base and derived unit constants.
#[allow(non_upper_case_globals)]
#[rustfmt::skip]
pub mod units {
    use super::{Dimension, Quantity, UNITD};
    // Base
    pub const s: Quantity = Quantity::unit(Dimension{s: 1, ..UNITD }, None);
    pub const m: Quantity = Quantity::unit(Dimension{m: 1, ..UNITD }, None);
    pub const kg: Quantity = Quantity::unit(Dimension{kg: 1, ..UNITD }, None);
    pub const A: Quantity = Quantity::unit(Dimension{A: 1, ..UNITD }, None);
    pub const K: Quantity = Quantity::unit(Dimension{K: 1, ..UNITD }, None);
    pub const mol: Quantity = Quantity::unit(Dimension{mol: 1, ..UNITD }, None);
    pub const cd: Quantity = Quantity::unit(Dimension{cd: 1, ..UNITD }, None);
    // Derived
    pub const rad: Quantity = Quantity::unit(Dimension{..UNITD }, Some(("rad", "radian")));
    pub const sr: Quantity = Quantity::unit(Dimension{..UNITD }, Some(("sr", "steradian")));
    pub const Hz: Quantity = Quantity::unit(Dimension{s: -1, ..UNITD }, None);
    pub const N: Quantity = Quantity::unit(Dimension{kg: 1, m: 1, s: -2, ..UNITD }, None);
    pub const Pa: Quantity = Quantity::unit(Dimension{kg: 1, m: -1, s: -2, ..UNITD }, None);
    pub const J: Quantity = Quantity::unit(Dimension{kg: 1, m: 2, s: -2, ..UNITD }, None);
    pub const W: Quantity = Quantity::unit(Dimension{kg: 1, m: 2, s: -3, ..UNITD }, None);
    pub const C: Quantity = Quantity::unit(Dimension{s: 1, A: 1, ..UNITD }, None);
    pub const V: Quantity = Quantity::unit(Dimension{s: -3, m: 2, kg: 1, A: -1, ..UNITD }, None);
    pub const F: Quantity = Quantity::unit(Dimension{s: 4, m: -2, kg: -1, A: 2, ..UNITD }, None);
    pub const OHM: Quantity = Quantity::unit(Dimension{s: -3, m: 2, kg: 1, A: -2, ..UNITD }, None);
    pub const S: Quantity = Quantity::unit(Dimension{s: 3, m: -2, kg: -1, A: 2, ..UNITD }, None);
    pub const Wb: Quantity = Quantity::unit(Dimension{s: -2, m: 2, kg: 1, A: -1, ..UNITD }, None);
    pub const T: Quantity = Quantity::unit(Dimension{s: -2, kg: 1, A: -1, ..UNITD }, None);
    pub const H: Quantity = Quantity::unit(Dimension{s: -2, m: 2, kg: 1, A: -2, ..UNITD }, None);
    pub const kat: Quantity = Quantity::unit(Dimension{s: -1, mol: 1, ..UNITD }, None);
}

#[cfg(test)]
mod tests {

    #[test]
    fn quantity_tostring() {
        use super::units::*;
        // Random
        assert_eq!((7.91 * m * m / s).to_string(), "7.91 m²·s⁻¹");
        // Force
        assert_eq!((3.2e-5 * kg * m / s / s).to_string(), "32 µN");
        // Frequency
        assert_eq!((1e8 / s).to_string(), "100 MHz");
        // Pressure
        assert_eq!((100.0 * N / m / m).to_string(), "100 Pa");
        assert_eq!((100.0 * kg / m / s / s).to_string(), "100 Pa");
        // Weights
        assert_eq!((100.0 * kg).to_string(), "100 kg");
        assert_eq!((0.1 * kg).to_string(), "100 g");
        assert_eq!((0.0001 * kg).to_string(), "100 mg");
        // Resistance
        assert_eq!(OHM.dimension.to_string(), "m²·kg·A⁻²·s⁻³");
        // Power
        assert_eq!((1e6 * J / s).to_string(), "1 MW");
        // Radian
        assert_eq!(((3.0 * m) / (1.0 * m)).to_string(), "3");
        assert_eq!(((3e5 * m) / (10.0 * m)).to_string(), "30 k");
    }

    #[test]
    fn magnitude_normalization() {
        use super::normalize;
        // small positives
        assert_eq!(normalize(3.2e-1).1, -3);
        assert_eq!(normalize(3.2e-4).1, -6);
        // small negatives
        assert_eq!(normalize(-3.2e-1).1, -3);
        assert_eq!(normalize(-3.2e-4).1, -6);
        // large positives
        assert_eq!(normalize(3.2e1).1, 0);
        assert_eq!(normalize(3.2e4).1, 3);
        // large negatives
        assert_eq!(normalize(-3.2e1).1, 0);
        assert_eq!(normalize(-3.2e4).1, 3);
    }

    #[test]
    fn scalar_mul_div() {
        use super::units::*;
        let v = 10.0 * m / s;
        assert_eq!((v * 2.0).to_string(), "20 m·s⁻¹");
        assert_eq!((v / 2.0).to_string(), "5 m·s⁻¹");
        assert_eq!((2.0 * v).to_string(), "20 m·s⁻¹");
    }

    #[test]
    fn scalar_div_quantity() {
        use super::units::*;
        assert_eq!((1.0 / (2.0 * s)).to_string(), "500 mHz");
        assert_eq!((2.0 / (1.0 * s)).to_string(), "2 Hz");
    }

    #[test]
    fn neg_quantity() {
        use super::units::*;
        let v = 5.0 * m;
        assert_eq!((-v).to_string(), "-5 m");
    }

    #[test]
    fn add_sub_quantity() {
        use super::units::*;
        let a = 5.0 * m;
        let b = 3.0 * m;
        assert_eq!((a + b).to_string(), "8 m");
        assert_eq!((a - b).to_string(), "2 m");
    }

    #[test]
    fn equality_and_order() {
        use super::units::*;
        let a = 5.0 * m;
        let b = 5.0 * m;
        let c = 3.0 * m;
        let d = 5.0 * s;
        assert_eq!(a, b);
        assert!(a > c);
        assert!(a.partial_cmp(&d).is_none());
    }

    #[test]
    fn getters() {
        use super::units::*;
        let q = 5.0 * m;
        assert_eq!(q.value(), 5.0);
        assert_eq!(q.dimension(), m.dimension());
        assert!(q.is_compatible_with(&(10.0 * m)));
        assert!(!q.is_compatible_with(&(10.0 * s)));
    }

    #[test]
    fn display_precision() {
        use super::units::*;
        let q = 1.23456 * m;
        assert_eq!(format!("{:.2}", q), "1.23 m");
    }

    #[test]
    fn display_precision_normalized() {
        use super::units::*;
        let q = 1.23456e3 * m;
        assert_eq!(format!("{:.2}", q), "1.23 km");
    }

    #[test]
    fn dimensionless_units() {
        use super::units::*;
        assert_eq!((3.0 * rad).to_string(), "3 rad");
        assert_eq!((2.0 * sr).to_string(), "2 sr");
        assert_eq!((rad + rad).to_string(), "2 rad");
        assert_eq!((rad * rad).to_string(), "1");
        assert_eq!(1.0 * rad, 1.0 * sr);
    }

    #[test]
    fn kat_dimension() {
        use super::units::*;
        assert_eq!(kat.dimension.to_string(), "mol·s⁻¹");
    }

    #[test]
    fn abs_quantity() {
        use super::units::*;
        let v = -5.0 * m / s;
        assert_eq!(v.abs().to_string(), "5 m·s⁻¹");
        assert_eq!((-3.0 * kg).abs().to_string(), "3 kg");
    }

    #[test]
    fn powi_quantity() {
        use super::units::*;
        let area = (2.0 * m).powi(2);
        assert_eq!(area.value(), 4.0);
        assert_eq!(area.dimension.to_string(), "m²");

        let freq = (2.0 * s).powi(-1);
        assert_eq!(freq.value(), 0.5);
        assert_eq!(freq.dimension.to_string(), "s⁻¹");
    }

    #[test]
    fn sqrt_quantity() {
        use super::units::*;
        let area = 4.0 * m * m;
        let side = area.sqrt();
        assert_eq!(side.value(), 2.0);
        assert_eq!(side.dimension.to_string(), "m");
    }

    #[test]
    #[should_panic]
    fn sqrt_odd_dimension_panics() {
        use super::units::*;
        let _ = (2.0 * m).sqrt();
    }

    #[test]
    fn in_unit_conversion() {
        use super::units::*;
        let dist = 500.0 * m;
        let km = 1000.0 * m;
        assert_eq!(dist.in_unit(km), 0.5);
        assert_eq!(dist.in_unit(m), 500.0);

        let mass = 0.1 * kg;
        let g = 0.001 * kg;
        assert_eq!(mass.in_unit(g), 100.0);

        let time = 3600.0 * s;
        let hour = 3600.0 * s;
        assert_eq!(time.in_unit(hour), 1.0);
    }

    #[test]
    #[should_panic]
    fn in_unit_mismatched_dimension_panics() {
        use super::units::*;
        let _ = (5.0 * m).in_unit(1.0 * s);
    }
}
