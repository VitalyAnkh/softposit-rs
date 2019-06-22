use super::PxE2;
use crate::WithSign;
use core::convert::From;
use core::f64;

impl<const N: u32> From<PxE2<{ N }>> for f32 {
    #[inline]
    fn from(a: PxE2<{ N }>) -> Self {
        f64::from(a) as f32
    }
}

impl<const N: u32> From<PxE2<{ N }>> for f64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_nar() {
            f64::NAN
        } else {
            let sign_a = PxE2::<{ N }>::sign_ui(ui_a);
            if sign_a {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = PxE2::<{ N }>::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 3) as u64) << 20;
            let exp_a = (((k_a as u64) << 2) + ((tmp >> 29) as u64)).wrapping_add(1023) << 52;

            f64::from_bits(exp_a + frac_a + (((sign_a as u64) & 0x1) << 63))
        }
    }
}

impl<const N: u32> From<f64> for PxE2<{ N }> {
    fn from(mut float: f64) -> Self {
        let mut reg: u32;
        let mut frac = 0_u32;
        let mut exp = 0_i32;
        let mut bit_n_plus_one = false;
        let mut bits_more = false;

        if float == 0. {
            return Self::ZERO;
        } else if !float.is_finite() {
            return Self::NAR;
        } /* else if float >= 1.329_227_995_784_916_e36 {
              //maxpos
              return Self::MAX;
          } else if float <= -1.329_227_995_784_916_e36 {
              // -maxpos
              return Self::MIN;
          }*/

        let sign = float < 0.;

        let u_z: u32 = if float == 1. {
            0x4000_0000
        } else if float == -1. {
            0xC000_0000
        /*} else if (float <= 7.523_163_845_262_64_e-37) && !sign {
            //minpos
            0x1
        } else if (float >= -7.523_163_845_262_64_e-37) && sign {
            //-minpos
            0xFFFF_FFFF*/
        } else if (float > 1.) || (float < -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }

            let reg_s = true;
            reg = 1; //because k = m-1; so need to add back 1
                     // minpos
            if (N == 2) && (float <= 7.523_163_845_262_64_e-37) {
                1
            } else {
                //regime
                while float >= 16. {
                    float *= 0.0625; // float/=16;
                    reg += 1;
                }
                while float >= 2. {
                    float *= 0.5;
                    exp += 1;
                }

                let frac_length = (N - 4) as isize - (reg as isize);

                if frac_length < 0 {
                    //in both cases, reg=29 and 30, e is n+1 bit and frac are sticky bits
                    if reg == N - 3 {
                        bit_n_plus_one = (exp & 0x1) != 0;
                        //exp>>=1; //taken care of by the pack algo
                        exp &= 0x2;
                    } else {
                        //reg=30
                        bit_n_plus_one = (exp >> 1) != 0;
                        bits_more = (exp & 0x1) != 0;
                        exp = 0;
                    }
                    if float != 1. {
                        //because of hidden bit
                        bits_more = true;
                        frac = 0;
                    }
                } else {
                    frac = crate::convert_fraction_p32(
                        float,
                        frac_length as u16,
                        &mut bit_n_plus_one,
                        &mut bits_more,
                    );
                }

                if reg > (N - 2) {
                    if reg_s {
                        0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32)
                    } else {
                        0x1 << (32 - N)
                    }
                } else {
                    //rounding off fraction bits

                    let regime = if reg_s { ((1 << reg) - 1) << 1 } else { 1_u32 };

                    if (N == 32) && (reg == 29) {
                        exp >>= 1;
                    } else if reg <= 28 {
                        exp <<= 28 - reg;
                    }

                    let mut u_z = ((regime as u32) << (30 - reg))
                        + (exp as u32)
                        + ((frac << (32 - N)) as u32);
                    //minpos
                    if (u_z == 0) && (frac > 0) {
                        u_z = 0x1 << (32 - N);
                    }
                    if bit_n_plus_one {
                        u_z += (((u_z >> (32 - N)) & 0x1) | (bits_more as u32)) << (32 - N);
                    }
                    u_z
                }
                .with_sign(sign)
            }
        } else if (float < 1.) || (float > -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }

            let reg_s = false;
            reg = 0;

            //regime
            while float < 1. {
                float *= 16.;
                reg += 1;
            }

            while float >= 2. {
                float *= 0.5;
                exp += 1;
            }

            let frac_length = (N - 4) as isize - (reg as isize);
            if frac_length < 0 {
                //in both cases, reg=29 and 30, e is n+1 bit and frac are sticky bits
                if reg == N - 3 {
                    bit_n_plus_one = (exp & 0x1) != 0;
                    //exp>>=1; //taken care of by the pack algo
                    exp &= 0x2;
                } else {
                    //reg=30
                    bit_n_plus_one = (exp >> 1) != 0;
                    bits_more = (exp & 0x1) != 0;
                    exp = 0;
                }
                if float != 1. {
                    //because of hidden bit
                    bits_more = true;
                    frac = 0;
                }
            } else {
                frac = crate::convert_fraction_p32(
                    float,
                    frac_length as u16,
                    &mut bit_n_plus_one,
                    &mut bits_more,
                );
            }

            if reg > (N - 2) {
                if reg_s {
                    0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32)
                } else {
                    0x1 << (32 - N)
                }
            } else {
                //rounding off fraction bits

                let regime = if reg_s { ((1 << reg) - 1) << 1 } else { 1_u32 };

                if (N == 32) && (reg == 29) {
                    exp >>= 1;
                } else if reg <= 28 {
                    exp <<= 28 - reg;
                }

                let mut u_z =
                    ((regime as u32) << (30 - reg)) + (exp as u32) + ((frac << (32 - N)) as u32);
                //minpos
                if (u_z == 0) && (frac > 0) {
                    u_z = 0x1 << (32 - N);
                }

                if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 0x1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
            .with_sign(sign)
        } else {
            //NaR - for NaN, INF and all other combinations
            0x8000_0000
        };
        Self::from_bits(u_z)
    }
}

impl<const N: u32> From<i32> for PxE2<{ N }> {
    #[inline]
    fn from(mut i_a: i32) -> Self {
        if i_a < -2147483135 {
            Self::from_bits(0x80500000);
        }

        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }

        let ui_a = if (N == 2) && (i_a > 0) {
            0x40000000
        } else if i_a > 2147483135 {
            //2147483136 to 2147483647 rounds to P32 value (2147483648)=> 0x7FB00000
            let mut ui_a = 0x7FB00000; // 2147483648
            if N < 10 {
                ui_a &= ((-0x80000000_i32) >> (N - 1)) as u32;
            } else if N < 12 {
                ui_a = 0x7FF00000 & (((-0x80000000_i32) >> (N - 1)) as u32);
            }
            ui_a
        } else {
            convert_u32_to_px2bits::<{ N }>(i_a as u32)
        };
        Self::from_bits(ui_a.with_sign(sign))
    }
}

impl<const N: u32> From<u32> for PxE2<{ N }> {
    #[inline]
    fn from(a: u32) -> Self {
        let ui_a = if (N == 2) && (a > 0) {
            0x40000000
        } else if a > 0xFFFFFBFF {
            //4294966271
            let mut ui_a = 0x7FC00000; // 4294967296
            if N < 12 {
                ui_a &= ((-0x80000000_i32) >> (N - 1)) as u32;
            }
            ui_a
        } else {
            convert_u32_to_px2bits::<{ N }>(a)
        };
        Self::from_bits(ui_a)
    }
}

fn convert_u32_to_px2bits<const N: u32>(a: u32) -> u32 {
    let mut log2 = 31_i8; //length of bit (e.g. 4294966271) in int (32 but because we have only 32 bits, so one bit off to accomdate that fact)
    let mut mask = 0x80000000_u32;
    if a < 0x2 {
        a << 30
    } else {
        let mut frac_a = a;

        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }
        let k = (log2 >> 2) as u32;
        let exp_a = (log2 & 0x3) as u32;
        frac_a ^= mask;

        let mut ui_a: u32;
        if k >= (N - 2) {
            //maxpos
            ui_a = 0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32);
        } else if k == (N - 3) {
            //bitNPlusOne-> first exp bit //bitLast is zero
            ui_a = 0x7FFFFFFF ^ (0x3FFFFFFF >> k);
            if ((exp_a & 0x2) != 0) && (((exp_a & 0x1) | frac_a) != 0) {
                //bitNPlusOne //bitsMore
                ui_a |= 0x80000000_u32 >> (N - 1);
            }
        } else if k == (N - 4) {
            ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | ((exp_a & 0x2) << (27 - k));
            if (exp_a & 0x1) != 0 {
                if (((0x80000000_u32 >> (N - 1)) & ui_a) | frac_a) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        } else if k == (N - 5) {
            ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | (exp_a << (27 - k));
            mask = 0x8 << (k - N);
            if (mask & frac_a) != 0 {
                //bitNPlusOne
                if (((mask - 1) & frac_a) | (exp_a & 0x1)) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        } else {
            ui_a = ((0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | (exp_a << (27 - k)) | frac_a >> (k + 4))
                & (((-0x80000000_i32) >> (N - 1)) as u32);;
            mask = 0x8 << (k - N); //bitNPlusOne
            if (mask & frac_a) != 0 {
                if (((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        }
        ui_a
    }
}

impl<const N: u32> From<i64> for PxE2<{ N }> {
    #[inline]
    fn from(mut i_a: i64) -> Self {
        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }

        let ui_a = if (N == 2) && (i_a > 0) {
            0x40000000
        } else if i_a > 0x7FFDFFFFFFFFFFFF {
            //9222809086901354495
            let mut ui_a = 0x7FFFB000; // P32: 9223372036854775808
            if N < 18 {
                ui_a &= ((-0x80000000_i32) >> (N - 1)) as u32;
            }
            ui_a
        } else {
            convert_u32_to_px2bits::<{ N }>(i_a as u32)
        };
        Self::from_bits(ui_a.with_sign(sign))
    }
}

impl<const N: u32> From<u64> for PxE2<{ N }> {
    #[inline]
    fn from(a: u64) -> Self {
        let ui_a = if (N == 2) && (a > 0) {
            0x40000000
        } else if a > 0xFFFBFFFFFFFFFFFF {
            //18445618173802708991
            let mut ui_a = 0x7FFFC000; // 18446744073709552000
            if N < 18 {
                ui_a &= ((-0x80000000_i32) >> (N - 1)) as u32;
            }
            ui_a
        } else {
            convert_u64_to_px2bits::<{ N }>(a)
        };
        Self::from_bits(ui_a)
    }
}

fn convert_u64_to_px2bits<const N: u32>(a: u64) -> u32 {
    let mut log2 = 63_i8; //length of bit (e.g. 18445618173802708991) in int (64 but because we have only 64 bits, so one bit off to accommodate that fact)
    let mut mask = 0x8000000000000000_u64;
    if a < 0x2 {
        (a as u32) << 30
    } else {
        let mut frac64_a = a;
        while (frac64_a & mask) == 0 {
            log2 -= 1;
            frac64_a <<= 1;
        }

        let k = (log2 >> 2) as u32;

        let exp_a = (log2 & 0x3) as u32;
        frac64_a ^= mask;

        let mut ui_a: u32;
        if k >= (N - 2) {
            //maxpos
            ui_a = 0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32);
        } else if k == (N - 3) {
            //bitNPlusOne-> first exp bit //bitLast is zero
            ui_a = 0x7FFFFFFF ^ (0x3FFFFFFF >> k);
            if ((exp_a & 0x2) != 0) && (((exp_a & 0x1) as u64 | frac64_a) != 0) {
                //bitNPlusOne //bitsMore
                ui_a |= 0x80000000_u32 >> (N - 1);
            }
        } else if k == (N - 4) {
            ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | ((exp_a & 0x2) << (27 - k));
            if (exp_a & 0x1) != 0 {
                if (((0x80000000_u32 >> (N - 1)) & ui_a) != 0) || (frac64_a != 0) {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        } else if k == (N - 5) {
            ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | (exp_a << (27 - k));
            mask = 0x800000000_u64 << (k + 32 - N);
            if (mask & frac64_a) != 0 {
                //bitNPlusOne
                if (((mask - 1) & frac64_a) | ((exp_a & 0x1) as u64)) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        } else {
            ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k))
                | (exp_a << (27 - k))
                | (((frac64_a >> (k + 36)) as u32) & (((-0x80000000_i32) >> (N - 1)) as u32));
            mask = 0x800000000_u64 << (k + 32 - N); //bitNPlusOne position
            if (mask & frac64_a) != 0 {
                if (((mask - 1) & frac64_a) | ((mask << 1) & frac64_a)) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        }
        ui_a
    }
}

use crate::P32E2;
impl<const N: u32> From<PxE2<{ N }>> for i32 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
    }
}

impl<const N: u32> From<PxE2<{ N }>> for u32 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
    }
}

impl<const N: u32> From<PxE2<{ N }>> for u64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
    }
}

impl<const N: u32> From<PxE2<{ N }>> for i64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
    }
}