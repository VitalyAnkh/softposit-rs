use super::P8E0;

const HALF: P8E0 = P8E0::new(0x_20);
const TWO: P8E0 = P8E0::new(0x_60);

impl crate::MathConsts for P8E0 {
    const E: Self = Self::new(0x_66);
    const FRAC_1_PI: Self = Self::new(0x_14);
    const FRAC_1_SQRT_2: Self = Self::new(0x_2d);
    const FRAC_2_PI: Self = Self::new(0x_29);
    const FRAC_2_SQRT_PI: Self = Self::new(0x_44);
    const FRAC_PI_2: Self = Self::new(0x_52);
    const FRAC_PI_3: Self = Self::new(0x_42);
    const FRAC_PI_4: Self = Self::new(0x_32);
    const FRAC_PI_6: Self = Self::new(0x_22);
    const FRAC_PI_8: Self = Self::new(0x_19);
    const LN_10: Self = Self::new(0x_62);
    const LN_2: Self = Self::new(0x_2c);
    const LOG10_E: Self = Self::new(0x_1c);
    const LOG2_E: Self = Self::new(0x_4e);
    const PI: Self = Self::new(0x_69);
    const SQRT_2: Self = Self::new(0x_4d);
    const LOG2_10: Self = Self::new(0x_6b);
    const LOG10_2: Self = Self::new(0x_13);
}

impl P8E0 {
    #[inline]
    pub fn trunc(self) -> Self {
        if self > Self::ZERO {
            self.floor()
        } else {
            self.ceil()
        }
    }
    #[inline]
    pub fn fract(self) -> Self {
        self - self.trunc()
    }
    #[inline]
    pub fn div_euclid(self, rhs: Self) -> Self {
        let q = (self / rhs).trunc();
        if self % rhs < Self::ZERO {
            return if rhs > Self::ZERO {
                q - Self::ONE
            } else {
                q + Self::ONE
            };
        }
        q
    }
    #[inline]
    pub fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < Self::ZERO {
            r + rhs.abs()
        } else {
            r
        }
    }
    #[inline]
    pub fn powi(self, _n: i32) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn powf(self, _n: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn exp2(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log2(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log10(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cbrt(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn hypot(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn acos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan2(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }
    #[inline]
    pub fn exp_m1(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn ln_1p(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sinh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cosh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tanh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asinh(self) -> Self {
        if self.is_nar() {
            self
        } else {
            (self + ((self * self) + Self::ONE).sqrt()).ln()
        }
    }
    #[inline]
    pub fn acosh(self) -> Self {
        match self {
            x if x < Self::ONE => Self::NAR,
            x => (x + ((x * x) - Self::ONE).sqrt()).ln(),
        }
    }
    #[inline]
    pub fn atanh(self) -> Self {
        HALF * ((TWO * self) / (Self::ONE - self)).ln_1p()
    }
}

mod ceil;
mod exp;
mod floor;
mod ln;
mod mul_add;
mod round;
mod sqrt;
