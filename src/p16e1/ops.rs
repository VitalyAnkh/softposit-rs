use super::P16E1;
use crate::WithSign;
use core::ops;

impl ops::Neg for P16E1 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }
}

impl ops::AddAssign for P16E1 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl ops::SubAssign for P16E1 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl ops::MulAssign for P16E1 {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl ops::DivAssign for P16E1 {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl ops::RemAssign for P16E1 {
    #[inline]
    fn rem_assign(&mut self, other: Self) {
        *self = *self % other
    }
}

impl P16E1 {
    #[inline]
    pub fn sub_mags(mut ui_a: u16, mut ui_b: u16) -> Self {
        //Both ui_a and ui_b are actually the same signs if ui_b inherits sign of sub
        //Make both positive
        let mut sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        } else {
            ui_b = ui_b.wrapping_neg();
        }

        if ui_a == ui_b {
            //essential, if not need special handling
            return Self::ZERO;
        }
        if ui_a < ui_b {
            ui_a ^= ui_b;
            ui_b ^= ui_a;
            ui_a ^= ui_b;
            sign = !sign; //A becomes B
        }

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac32_a = (frac_a as u32) << 16;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        let mut frac32_b = (frac_b as u32) << 16;

        let mut shift_right = (k_a as i16) - (k_b as i16);

        //This is 2kZ + expZ; (where kZ=k_a-k_b and expZ=exp_a-expB)

        shift_right = (shift_right << 1) + (exp_a as i16) - (exp_b as i16);

        if shift_right != 0 {
            if shift_right >= 29 {
                let mut u_z: u16 = ui_a;
                if sign {
                    u_z = (-(u_z as i16)) as u16/* & 0xFFFF*/;
                }
                return Self::from_bits(u_z);
            } else {
                frac32_b >>= shift_right;
            }
        }

        frac32_a -= frac32_b;

        while (frac32_a >> 29) == 0 {
            k_a -= 1;
            frac32_a <<= 2;
        }
        let ecarry = (0x4000_0000 & frac32_a) >> 30 != 0;
        if !ecarry {
            if exp_a == 0 {
                k_a -= 1;
            }
            exp_a ^= 1;
            frac32_a <<= 1;
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove hidden bits
            frac32_a = (frac32_a & 0x3FFF_FFFF) >> (reg_a + 1);
            let mut frac_a = (frac32_a >> 16) as u16;
            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = ((frac32_a >> 15) & 0x1) != 0;
            } else if frac32_a > 0 {
                frac_a = 0;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }
            let mut u_z = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
            if bit_n_plus_one {
                let bits_more = (frac32_a & 0x7FFF) != 0;
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };
        Self::from_bits(u_z.with_sign(sign))
    }

    #[inline]
    pub fn add_mags(mut ui_a: u16, mut ui_b: u16) -> Self {
        let sign = Self::sign_ui(ui_a); //sign is always positive.. actually don't have to do this.
        if sign {
            ui_a = ui_a.wrapping_neg();
            ui_b = ui_b.wrapping_neg();
        }

        if (ui_a as i16) < (ui_b as i16) {
            let ui_x = ui_a;
            let ui_y = ui_b;
            ui_a = ui_y;
            ui_b = ui_x;
        }

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac32_a = (frac_a as u32) << 16;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        let mut frac32_b = (frac_b as u32) << 16;

        let mut shift_right = (k_a as i16) - (k_b as i16);

        //This is 2kZ + expZ; (where kZ=k_a-k_b and expZ=exp_a-expB)
        shift_right = (shift_right << 1) + (exp_a as i16) - (exp_b as i16);

        if shift_right == 0 {
            frac32_a += frac32_b;
            //rcarry is one
            if exp_a != 0 {
                k_a += 1;
            }
            exp_a ^= 1;
            frac32_a >>= 1;
        } else {
            //Manage CLANG (LLVM) compiler when shifting right more than number of bits
            if shift_right > 31 {
                frac32_b = 0;
            } else {
                //frac32_b >>= shift_right
                frac32_b >>= shift_right;
            }

            frac32_a += frac32_b;
            let rcarry = (frac32_a & 0x8000_0000) != 0; //first left bit
            if rcarry {
                if exp_a != 0 {
                    k_a += 1;
                }
                exp_a ^= 1;
                frac32_a >>= 1;
            }
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove hidden bits
            frac32_a = (frac32_a & 0x3FFF_FFFF) >> (reg_a + 1);
            let mut frac_a = (frac32_a >> 16) as u16;
            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = ((frac32_a >> 15) & 0x1) != 0;
            } else if frac32_a > 0 {
                frac_a = 0;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }
            let mut u_z = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
            if bit_n_plus_one {
                let bits_more = (frac32_a & 0x7FFF) != 0;
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };
        Self::from_bits(u_z.with_sign(sign))
    }
}

impl ops::Add for P16E1 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //Zero or infinity
        if self.is_zero() || other.is_zero() {
            // Not required but put here for speed
            Self::from_bits(ui_a | ui_b)
        } else if self.is_nar() || other.is_nar() {
            Self::NAR
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                Self::sub_mags(ui_a, ui_b)
            } else {
                Self::add_mags(ui_a, ui_b)
            }
        }
    }
}

impl ops::Sub for P16E1 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //infinity
        if self.is_nar() || other.is_nar() {
            Self::NAR
        } else if self.is_zero() || other.is_zero() {
            //Zero
            Self::from_bits(ui_a | ui_b.wrapping_neg())
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                Self::add_mags(ui_a, ui_b.wrapping_neg())
            } else {
                Self::sub_mags(ui_a, ui_b.wrapping_neg())
            }
        }
    }
}

impl ops::Mul for P16E1 {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if self.is_nar() || other.is_nar() {
            return Self::NAR;
        } else if self.is_zero() || other.is_zero() {
            return Self::ZERO;
        }

        let sign_a = Self::sign_ui(ui_a);
        let sign_b = Self::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;

        if sign_a {
            ui_a = ui_a.wrapping_neg();
        }
        if sign_b {
            ui_b = ui_b.wrapping_neg();
        }

        let (mut k_a, mut exp_a, mut frac_a) = P16E1::separate_bits(ui_a);
        let (k_b, exp_b, frac_b) = P16E1::separate_bits(ui_b);
        k_a += k_b;
        exp_a += exp_b;
        let mut frac32_z = (frac_a as u32) * (frac_b as u32);

        if exp_a > 1 {
            k_a += 1;
            exp_a ^= 0x2;
        }

        let rcarry = (frac32_z >> 29) != 0; //3rd bit of frac32_z
        if rcarry {
            if exp_a != 0 {
                k_a += 1;
            }
            exp_a ^= 1;
            frac32_z >>= 1;
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac32_z = (frac32_z & 0xFFF_FFFF) >> (reg_a - 1);
            frac_a = (frac32_z >> 16) as u16;

            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = (0x8000 & frac32_z) != 0;
            } else if frac_a > 0 {
                frac_a = 0;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }

            //sign is always zero
            let mut u_z = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (0x7FFF & frac32_z) != 0;
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl ops::Div for P16E1 {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if self.is_nar() || other.is_nar() || other.is_zero() {
            return Self::NAR;
        } else if self.is_zero() {
            return Self::ZERO;
        }

        let sign_a = Self::sign_ui(ui_a);
        let sign_b = Self::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;

        if sign_a {
            ui_a = ui_a.wrapping_neg();
        }
        if sign_b {
            ui_b = ui_b.wrapping_neg();
        }

        let (mut k_a, mut exp_a, mut frac_a) = Self::separate_bits(ui_a);
        let frac32_a = (frac_a as u32) << 14;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        k_a -= k_b;
        exp_a -= exp_b;

        let (quot, rem) = crate::div(frac32_a as i32, frac_b as i32);
        let mut frac32_z = quot as u32;

        if exp_a < 0 {
            exp_a = 1;
            k_a -= 1;
        }
        if frac32_z != 0 {
            let rcarry = (frac32_z >> 14) != 0; // this is the hidden bit (14th bit) , extreme right bit is bit 0
            if !rcarry {
                if exp_a == 0 {
                    k_a -= 1;
                }
                exp_a ^= 1;
                frac32_z <<= 1;
            }
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac32_z &= 0x3FFF;
            frac_a = (frac32_z >> (reg_a + 1)) as u16;

            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = ((frac32_z >> reg_a) & 0x1) != 0;
            } else if frac_a > 0 {
                frac_a = 0;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }

            //sign is always zero
            let mut u_z = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);

            if bit_n_plus_one {
                let bits_more = if rem != 0 {
                    true
                } else {
                    (((1 << reg_a) - 1) & frac32_z) != 0
                };
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl ops::Rem for P16E1 {
    type Output = Self;
    fn rem(self, _other: Self) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
fn test_ops(fun: fn(P16E1, P16E1, f64, f64) -> (P16E1, f64)) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let (p, f) = fun(p_a, p_b, f_a, f_b);
        assert_eq!(p, P16E1::from(f));
    }
}

#[test]
fn add() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a + p_b, f_a + f_b));
}

#[test]
fn sub() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a - p_b, f_a - f_b));
}

#[test]
fn mul() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a * p_b, f_a * f_b));
}

#[test]
fn div() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a / p_b, f_a / f_b));
}
