use super::PxE1;
use crate::WithSign;
use core::{mem, ops};

impl<const N: u32> ops::Neg for PxE1<{ N }> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }
}

impl<const N: u32> ops::AddAssign for PxE1<{ N }> {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl<const N: u32> ops::SubAssign for PxE1<{ N }> {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl<const N: u32> ops::MulAssign for PxE1<{ N }> {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl<const N: u32> ops::DivAssign for PxE1<{ N }> {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl<const N: u32> ops::Add for PxE1<{ N }> {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //Zero or infinity
        if (ui_a == 0) || (ui_b == 0) {
            // Not required but put here for speed
            Self::from_bits(ui_a | ui_b)
        } else if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
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

impl<const N: u32> ops::Sub for PxE1<{ N }> {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //infinity
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            Self::NAR
        } else if (ui_a == 0) || (ui_b == 0) {
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

impl<const N: u32> PxE1<{ N }> {
    #[inline]
    fn add_mags(mut ui_a: u32, mut ui_b: u32) -> Self {
        let sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
            ui_b = ui_b.wrapping_neg();
        }

        if (ui_a as i32) < (ui_b as i32) {
            mem::swap(&mut ui_a, &mut ui_b);
        }

        let u_z = if N == 2 {
            let reg_sa = Self::sign_reg_ui(ui_a);
            let reg_sb = Self::sign_reg_ui(ui_b);
            if reg_sa | reg_sb {
                0x_4000_0000
            } else {
                0x0
            }
        } else {
            let (mut k_a, mut exp, frac) = Self::separate_bits(ui_a);

            let mut frac64 = (frac as u64) << 32;

            let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);

            let mut shift_right = (k_a as i16) - (k_b as i16);
            let mut frac64_b = (frac_b as u64) << 32;

            //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp-expB)
            shift_right = (shift_right << 2) + (exp as i16) - (exp_b as i16);

            if shift_right == 0 {
                frac64 += frac64_b;
                //rcarry is one
                if exp != 0 {
                    k_a += 1;
                }
                exp ^= 1;
                frac64 >>= 1;
            } else {
                //Manage CLANG (LLVM) compiler when shifting right more than number of bits
                if shift_right > 63 {
                    frac64_b = 0
                } else {
                    frac64_b >>= shift_right
                }; //frac64B >>= shiftRight

                frac64 += frac64_b;

                let rcarry = (0x_8000_0000_0000_0000 & frac64) != 0; //first left bit
                if rcarry {
                    if exp != 0 {
                        k_a += 1;
                    }
                    exp ^= 1;
                    frac64 >>= 1;
                }
            }

            let (mut regime, reg_s, reg_len) = Self::calculate_regime(k_a);

            if reg_len > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_s {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                //remove hidden bits
                frac64 = (frac64 & 0x_3FFF_FFFF_FFFF_FFFF) >> (reg_len + 1); // 2 bits exp

                //regime length is smaller than length of posit
                let mut bit_n_plus_one = false;
                let frac = if reg_len < N {
                    if reg_len != (N - 2) {
                        bit_n_plus_one = ((0x_8000_0000_0000_0000_u64 >> N) & frac64) != 0;
                        ((frac64 >> 32) as u32) & Self::mask()
                    } else {
                        bit_n_plus_one = exp != 0;
                        exp = 0;
                        0
                    }
                } else {
                    if reg_s {
                        regime &= Self::mask()
                    } else {
                        regime <<= 32 - N
                    }
                    exp = 0;
                    0
                };

                exp <<= 29 - reg_len;
                let mut u_z = Self::pack_to_ui(regime, exp as u32, frac);

                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                if bit_n_plus_one {
                    let bits_more = ((0x_FFFF_FFFF_FFFF_FFFF_u64 >> N) & frac64) != 0;
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
        };

        Self::from_bits(u_z.with_sign(sign))
    }

    #[inline]
    fn sub_mags(mut ui_a: u32, mut ui_b: u32) -> Self {
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

        if (ui_a as i32) < (ui_b as i32) {
            mem::swap(&mut ui_a, &mut ui_b);
            sign = !sign; //A becomes B
        }

        let u_z = if N == 2 {
            let reg_sa = Self::sign_reg_ui(ui_a);
            let reg_sb = Self::sign_reg_ui(ui_b);
            if reg_sa == reg_sb {
                0
            } else {
                0x_4000_0000
            }
        } else {
            let (mut k_a, mut exp, frac) = Self::separate_bits(ui_a);
            let mut frac64 = (frac as u64) << 32;

            let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);

            let mut shift_right = (k_a as i16) - (k_b as i16);
            let mut frac64_b = (frac_b as u64) << 32;

            //This is 4kZ + expZ; (where kZ=kA-kB and expZ=exp-expB)
            shift_right = (shift_right << 2) + (exp as i16) - (exp_b as i16);

            if shift_right > 60 {
                return Self::from_bits(if sign { ui_a.wrapping_neg() } else { ui_a });
            } else {
                frac64_b >>= shift_right;
            }

            frac64 -= frac64_b;

            while (frac64 >> 61) == 0 {
                k_a -= 1;
                frac64 <<= 2;
            }
            let ecarry = (0x4000_0000_0000_0000 & frac64) != 0; //(0x4000000000000000 & frac64)>>62;
            if !ecarry {
                if exp == 0 {
                    k_a -= 1;
                }
                exp ^= 1;
                frac64 <<= 1;
            }

            let (mut regime, reg_s, reg_len) = Self::calculate_regime(k_a);

            if reg_len > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_s {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                //remove hidden bits
                frac64 = (frac64 & 0x_3FFF_FFFF_FFFF_FFFF) >> (reg_len + 1); // 2 bits exp

                //regime length is smaller than length of posit
                let mut bit_n_plus_one = false;
                let frac = if reg_len < N {
                    if reg_len != (N - 2) {
                        bit_n_plus_one = ((0x_8000_0000_0000_0000_u64 >> N) & frac64) != 0;
                        ((frac64 >> 32) as u32) & Self::mask()
                    } else {
                        bit_n_plus_one = exp != 0;
                        exp = 0;
                        0
                    }
                } else {
                    if reg_s {
                        regime &= Self::mask()
                    } else {
                        regime <<= 32 - N
                    }
                    exp = 0;
                    0
                };

                exp <<= 29 - reg_len;
                let mut u_z = Self::pack_to_ui(regime, exp as u32, frac);

                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                if bit_n_plus_one {
                    let bits_more = ((0x_FFFF_FFFF_FFFF_FFFF_u64 >> N) & frac64) != 0;
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
        };

        Self::from_bits(u_z.with_sign(sign))
    }
}

impl<const N: u32> ops::Mul for PxE1<{ N }> {
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
            ui_a = ui_a.wrapping_neg()
        };
        if sign_b {
            ui_b = ui_b.wrapping_neg()
        };

        let u_z = if N == 2 {
            let reg_sa = Self::sign_reg_ui(ui_a);
            let reg_sb = Self::sign_reg_ui(ui_b);
            if reg_sa & reg_sb {
                0x_4000_0000
            } else {
                0x0
            }
        } else {
            let (mut k_a, mut exp, frac_a) = Self::separate_bits(ui_a);

            let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
            k_a += k_b;
            exp += exp_b;
            let mut frac64 = (frac_a as u64) * (frac_b as u64);

            if exp > 1 {
                k_a += 1;
                exp ^= 0x2;
            }

            let rcarry = (frac64 >> 61) != 0; //3rd bit of frac64
            if rcarry {
                if exp != 0 {
                    k_a += 1;
                }
                exp ^= 1;
                frac64 >>= 1;
            }

            let (mut regime, reg_s, reg_len) = Self::calculate_regime(k_a);

            if reg_len > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_s {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                //remove carry and rcarry bits and shift to correct position (2 bits exp, so + 1 than 16 bits)
                frac64 = (frac64 & 0x_0FFF_FFFF_FFFF_FFFF) >> (reg_len - 1);

                //regime length is smaller than length of posit
                let mut bit_n_plus_one = false;
                let mut bits_more = false;
                let frac = if reg_len < N {
                    if reg_len != (N - 2) {
                        bit_n_plus_one = ((0x_8000_0000_0000_0000_u64 >> N) & frac64) != 0;
                        bits_more = ((0x_7FFF_FFFF_FFFF_FFFF >> N) & frac64) != 0;
                        ((frac64 >> 32) as u32) & Self::mask()
                    } else {
                        bit_n_plus_one = exp != 0;
                        bits_more = frac64 != 0;
                        exp = 0;
                        0
                    }
                } else {
                    regime = if reg_s {
                        regime & Self::mask()
                    } else {
                        regime << (32 - N)
                    };
                    exp = 0;
                    0
                };

                exp <<= 29 - reg_len;
                let mut u_z = Self::pack_to_ui(regime, exp as u32, frac);

                if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }

                u_z
            }
        };
        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl<const N: u32> ops::Div for PxE1<{ N }> {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //Zero or infinity
        if self.is_nar() || other.is_nar() || other.is_zero() {
            return Self::NAR;
        } else if self.is_zero() {
            return Self::ZERO;
        }

        let sign_a = Self::sign_ui(ui_a);
        let sign_b = Self::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;

        if sign_a {
            ui_a = ui_a.wrapping_neg()
        };
        if sign_b {
            ui_b = ui_b.wrapping_neg()
        };

        let u_z = if N == 2 {
            0x_4000_0000
        } else {
            let (mut k_a, mut exp, frac_a) = Self::separate_bits(ui_a);

            let frac64_a = (frac_a as u64) << 30;

            let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
            k_a -= k_b;
            exp -= exp_b;

            let (quot, rem) = crate::lldiv(frac64_a as i64, frac_b as i64);
            let mut frac64 = quot as u64;

            if exp < 0 {
                exp = 1;
                k_a -= 1;
            }
            if frac64 != 0 {
                let rcarry = (frac64 >> 30) != 0; // this is the hidden bit (14th bit) , extreme right bit is bit 0
                if !rcarry {
                    if exp == 0 {
                        k_a -= 1;
                    }
                    exp ^= 1;
                    frac64 <<= 1;
                }
            }

            let (mut regime, reg_s, reg_len) = Self::calculate_regime(k_a);

            if reg_len > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_s {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                //remove carry and rcarry bits and shift to correct position
                let frac64 = (frac64 & 0x_3FFF_FFFF) as u32;

                //regime length is smaller than length of posit
                let mut bit_n_plus_one = false;
                let mut bits_more = false;
                let frac;
                if reg_len < N {
                    if reg_len != (N - 2) {
                        bit_n_plus_one =
                            (((0x_8000_0000_u64 >> (N - reg_len - 1)) as u32) & frac64) != 0;
                        bits_more = ((0x_7FFF_FFFF >> (N - reg_len - 1)) & frac64) != 0;
                        frac = (frac64 >> (reg_len + 1)) & Self::mask();
                    } else {
                        bit_n_plus_one = exp != 0;
                        bits_more = frac64 > 0;
                        exp = 0;
                        frac = 0;
                    }
                    if rem != 0 {
                        bits_more = true;
                    }
                } else {
                    regime = if reg_s {
                        regime & Self::mask()
                    } else {
                        regime << (32 - N)
                    };
                    exp = 0;
                    frac = 0;
                }

                exp <<= 29 - reg_len;
                let mut u_z = Self::pack_to_ui(regime, exp as u32, frac);

                if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }

                u_z
            }
        };
        Self::from_bits(u_z.with_sign(sign_z))
    }
}
