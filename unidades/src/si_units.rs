// Ref: https://en.wikipedia.org/wiki/International_System_of_Units
// Quantity: the whole thing
// Value: the number
// Prefix: mega, kilo, etc.
// Unit: second, metre, etc

use std::ops;
use std::fmt;

const fn magnitude_prefix(factor: i32) -> Option<(&'static str, &'static str)> {
    Some(match factor {
        -24 => ("y", "yocto"),
        -21 => ("z", "zepto"),
        -18 => ("a", "atto"),
        -15 => ("f", "femto"),
        -12 => ("p", "pico"),
        -9 => ("n", "nano"),
        -6 => ("μ", "micro"),
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
        _ => return None
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

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Dimension {
    s: i8,
    m: i8,
    kg: i8,
    A: i8,
    K: i8,
    mol: i8,
    cd: i8,
}

impl Dimension {
    const fn names(&self) -> Option<(&'static str, &'static str)> {
        Some(match self {
            // Base
            Dimension{s: 1, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 0} => ("s", "second"),
            Dimension{s: 0, m: 1, kg: 0, A: 0, K: 0, mol: 0, cd: 0} => ("m", "meter"),
            Dimension{s: 0, m: 0, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("kg", "kilogram"),
            Dimension{s: 0, m: 0, kg: 0, A: 1, K: 0, mol: 0, cd: 0} => ("A", "ampere"),
            Dimension{s: 0, m: 0, kg: 0, A: 0, K: 1, mol: 0, cd: 0} => ("K", "kelvin"),
            Dimension{s: 0, m: 0, kg: 0, A: 0, K: 0, mol: 1, cd: 0} => ("mol", "mole"),
            Dimension{s: 0, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 1} => ("cd", "candela"),
            // Derived
            Dimension{s: -1, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 0} => ("Hz", "hertz"),
            Dimension{s: -2, m: 1, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("N", "newton"),
            Dimension{s: -2, m: -1, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("Pa", "pascal"),
            Dimension{s: -2, m: 2, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("J", "joule"),
            Dimension{s: -3, m: 2, kg: 1, A: 0, K: 0, mol: 0, cd: 0} => ("W", "watt"),
            Dimension{s: 1, m: 0, kg: 0, A: 1, K: 0, mol: 0, cd: 0} => ("C", "coulomb"),
            Dimension{s: -3, m: 2, kg: 1, A: -1, K: 0, mol: 0, cd: 0} => ("V", "volt"),
            Dimension{s: 4, m: -2, kg: -1, A: 2, K: 0, mol: 0, cd: 0} => ("F", "farad"),
            Dimension{s: -3, m: 2, kg: 1, A: -2, K: 0, mol: 0, cd: 0} => ("Ω", "ohm"),
            Dimension{s: 3, m: -2, kg: -1, A: 2, K: 0, mol: 0, cd: 0} => ("S", "siemens"),
            Dimension{s: -2, m: 2, kg: 1, A: -1, K: 0, mol: 0, cd: 0} => ("Wb", "weber"),
            Dimension{s: -2, m: 0, kg: 1, A: -1, K: 0, mol: 0, cd: 0} => ("T", "tesla"),
            Dimension{s: -2, m: 2, kg: 1, A: -2, K: 0, mol: 0, cd: 0} => ("H", "henry"),
            _ => return None
        })
    }
}


impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        static SUP: &[&str] = &[
            "\u{2070}", "\u{00b9}", "\u{00b2}", "\u{00b3}", "\u{2074}",
            "\u{2075}", "\u{2076}", "\u{2077}", "\u{2078}", "\u{2079}",
        ];
        macro_rules! fmtunit {
            ($dimension:ident, $dim_name:literal) => {match self.$dimension {
                0 => None,
                1 => Some($dim_name.to_string()),
                n => Some(if n > 1 {
                    format!("{}{}", $dim_name, SUP[n as usize])
                } else {
                    format!("{}\u{207b}{}", $dim_name, SUP[-n as usize])
                })
            }}
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
        write!(f, "{}", dims.into_iter()
            .filter_map(|x| x.1).collect::<Vec<_>>().join("\u{00b7}"))
    }
}

const UNITD: Dimension = Dimension{
    s: 0, m: 0, kg: 0, A: 0, K: 0, mol: 0, cd: 0
};

impl ops::Mul for Dimension {
    type Output = Dimension;
    fn mul(self, rhs: Dimension) -> Dimension {
        Dimension{
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
        Dimension{
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

#[derive(Clone, Copy)]
pub struct Quantity {
    value: f64,
    dimension: Dimension,
}

impl ops::Mul<Quantity> for f64 {
    type Output = Quantity;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        Quantity{value: self * rhs.value, dimension: rhs.dimension}
    }
}

impl ops::Div<Quantity> for f64 {
    type Output = Quantity;
    fn div(self, rhs: Self::Output) -> Self::Output {
        Quantity{value: self / rhs.value, dimension: UNITD / rhs.dimension}
    }
}

impl ops::Mul<Quantity> for Quantity {
    type Output = Quantity;
    fn mul(self, rhs: Self::Output) -> Self::Output {
        Quantity{
            value: self.value * rhs.value,
            dimension: self.dimension * rhs.dimension,
        }
    }
}

impl ops::Div<Quantity> for Quantity {
    type Output = Quantity;
    fn div(self, rhs: Self::Output) -> Self::Output {
        Quantity{
            value: self.value / rhs.value,
            dimension: self.dimension / rhs.dimension,
        }
    }
}

impl ops::Add<Quantity> for Quantity {
    type Output = Quantity;
    fn add(self, rhs: Self::Output) -> Self::Output {
        assert_eq!(self.dimension, rhs.dimension);
        Quantity{
            value: self.value + rhs.value,
            dimension: self.dimension,
        }
    }
}

impl ops::Sub<Quantity> for Quantity {
    type Output = Quantity;
    fn sub(self, rhs: Self::Output) -> Self::Output {
        assert_eq!(self.dimension, rhs.dimension);
        Quantity{
            value: self.value - rhs.value,
            dimension: self.dimension,
        }
    }
}

impl Quantity {
    const fn unit(dimension: Dimension) -> Quantity {
        Quantity{value: 1.0, dimension}
    }

    pub fn symbol(&self) -> String {
        self.dimension.names()
            .map(|x| x.0.to_string())
            .or(Some(self.dimension.to_string()))
            .unwrap()
    }

    pub fn name(&self) -> Option<String> {
        self.dimension.names()
            .map(|x| x.1.to_string())
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name().is_some() {

            if self.dimension == units::kg.dimension {
                let (value, factor) = normalize(self.value * 1000.0);
                write!(f, "{} {}g", value, magnitude_prefix(factor).unwrap().0)
            } else {
                let (value, factor) = normalize(self.value);
                write!(f, "{} {}{}",
                    value, magnitude_prefix(factor).unwrap().0, self.symbol())
            }
        } else {
            write!(f, "{} {}", self.value, self.symbol())
        }
    }
}


#[allow(non_upper_case_globals)]
pub mod units {
    use super::{Quantity, Dimension, UNITD};
    // Base
    pub const s: Quantity = Quantity::unit(Dimension{s: 1, ..UNITD});
    pub const m: Quantity = Quantity::unit(Dimension{m: 1, ..UNITD});
    pub const kg: Quantity = Quantity::unit(Dimension{kg: 1, ..UNITD});
    pub const A: Quantity = Quantity::unit(Dimension{A: 1, ..UNITD});
    pub const K: Quantity = Quantity::unit(Dimension{K: 1, ..UNITD});
    pub const mol: Quantity = Quantity::unit(Dimension{mol: 1, ..UNITD});
    pub const cd: Quantity = Quantity::unit(Dimension{cd: 1, ..UNITD});
    // Deriverd
    pub const rad: Quantity = Quantity::unit(Dimension{..UNITD});
    pub const sr: Quantity = Quantity::unit(Dimension{..UNITD});
    pub const Hz: Quantity = Quantity::unit(Dimension{s: -1, ..UNITD});
    pub const N: Quantity = Quantity::unit(Dimension{kg: 1, m: 1, s: -2, ..UNITD});
    pub const Pa: Quantity = Quantity::unit(Dimension{kg: 1, m: -1, s: -2, ..UNITD});
    pub const J: Quantity = Quantity::unit(Dimension{kg: 1, m: 2, s: -2, ..UNITD});
    pub const W: Quantity = Quantity::unit(Dimension{kg: 1, m: 2, s: -3, ..UNITD});
    pub const C: Quantity = Quantity::unit(Dimension{s: 1, A: 1, ..UNITD});
    pub const V: Quantity = Quantity::unit(Dimension{s: -3, m: 2, kg: 1, A: -1, ..UNITD});
    pub const F: Quantity = Quantity::unit(Dimension{s: 4, m: -2, kg: -1, A: 2, ..UNITD});
    pub const ohm: Quantity = Quantity::unit(Dimension{s: -3, m: 2, kg: 1, A: -2, ..UNITD});
    pub const S: Quantity = Quantity::unit(Dimension{s: 3, m: -2, kg: -1, A: 2, ..UNITD});
    pub const Wb: Quantity = Quantity::unit(Dimension{s: -2, m: 2, kg: 1, A: -1, ..UNITD});
    pub const T: Quantity = Quantity::unit(Dimension{s: -2,  kg: 1, A: -1, ..UNITD});
    pub const H: Quantity = Quantity::unit(Dimension{s: -2, m: 2, kg: 1, A: -2, ..UNITD});
}


#[cfg(test)]
mod tests {

    #[test]
    fn x() {
        use super::units::*;
        println!("resistance symbol: {} dimension: {}", ohm.symbol(), ohm.dimension);
        println!("gravity {}", 9.81 * m * m / s);
        println!("force {}", 3.2e-5 * kg * m / s / s);
        println!("freq {}", 1e8 / s);
        println!("pressure {}", 100.0 * kg / m / s / s);
        println!("weights {}, {}, {}", 100.0 * kg, 0.1 * kg, 0.0001 * kg);
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
}
