use super::PxE2;

impl<const N: u32> PxE2<{ N }> {
    #[inline]
    pub fn sqrt(self) -> Self {
        let mut ui_a = self.to_bits();

        // If NaR or a negative number, return NaR.
        if (ui_a & 0x8000_0000) != 0 {
            return Self::NAR;
        }
        // If the argument is zero, return zero.
        else if ui_a == 0 {
            return self;
        }
        // Compute the square root; shift_z is the power-of-2 scaling of the result.
        // Decode regime and exponent; scale the input to be in the range 1 to 4:
        let mut shift_z: i32;
        if (ui_a & 0x4000_0000) != 0 {
            shift_z = -2;
            while (ui_a & 0x4000_0000) != 0 {
                shift_z += 2;
                ui_a <<= 1 /*() & 0xFFFF_FFFF*/;
            }
        } else {
            shift_z = 0;
            while (ui_a & 0x4000_0000) == 0 {
                shift_z -= 2;
                ui_a <<= 1 /*90 & 0xFFFF_FFFF*/;
            }
        }

        ui_a &= 0x3FFF_FFFF;
        let mut exp_a = ui_a >> 28;
        shift_z += (exp_a >> 1) as i32;
        exp_a = 0x1 ^ (exp_a & 0x1);
        ui_a &= 0x0FFF_FFFF;
        let frac_a = ui_a | 0x1000_0000;

        // Use table look-up of first 4 bits for piecewise linear approx. of 1/sqrt:
        let index = (((frac_a >> 24) & 0xE) + exp_a) as usize;
        let eps = ((frac_a >> 9) & 0xFFFF) as i32;
        let r0: u32 = (crate::APPROX_RECIP_SQRT0[index] as u32)
            - (((crate::APPROX_RECIP_SQRT1[index] as u32) * (eps as u32)) >> 20);

        // Use Newton-Raphson refinement to get 33 bits of accuracy for 1/sqrt:
        let mut e_sqr_r0 = (r0 as u64) * (r0 as u64);
        if exp_a == 0 {
            e_sqr_r0 <<= 1;
        }
        let sigma0: u64 = 0xFFFF_FFFF & (0xFFFF_FFFF ^ ((e_sqr_r0 * (frac_a as u64)) >> 20));
        let mut recip_sqrt: u64 = ((r0 as u64) << 20) + (((r0 as u64) * sigma0) >> 21);

        let sqr_sigma0 = (sigma0 * sigma0) >> 35;
        recip_sqrt += ((recip_sqrt + (recip_sqrt >> 2) - ((r0 as u64) << 19)) * sqr_sigma0) >> 46;

        let mut frac64_z = ((frac_a as u64).wrapping_mul(recip_sqrt)) >> 31;
        if exp_a != 0 {
            frac64_z >>= 1;
        }

        // Find the exponent of Z and encode the regime bits.
        let exp_z = (shift_z as u32) & 0x3;
        let shift: u32;
        let ui_z: u32 = if shift_z < 0 {
            shift = ((-1 - shift_z) >> 2) as u32;
            0x2000_0000 >> shift
        } else {
            shift = (shift_z >> 2) as u32;
            0x7FFF_FFFF - (0x3FFF_FFFF >> shift)
        };

        // Trick for eliminating off-by-one cases that only uses one multiply:
        frac64_z += 1;

        if (frac64_z & 0xF) == 0 {
            let shifted_frac64_z = frac64_z >> 1;
            let neg_rem = (shifted_frac64_z * shifted_frac64_z) & 0x1_FFFF_FFFF;
            if (neg_rem & 0x1_0000_0000) != 0 {
                frac64_z |= 1;
            } else if neg_rem != 0 {
                frac64_z -= 1;
            }
        }
        // Strip off the hidden bit and round-to-nearest using last shift+5 bits.
        frac64_z &= 0xFFFFFFFF;
        let mask = 1 << (36 + shift - N);
        let u_a = if (mask & frac64_z) != 0 {
            if (((mask - 1) & frac64_z) | ((mask << 1) & frac64_z)) != 0 {
                frac64_z += mask << 1;
            }
            // Assemble the result and return it.
            ui_z | (exp_z << (27 - shift)) | ((frac64_z >> (5 + shift)) as u32)
        } else {
            // Assemble the result and return it.
            let mut u_a = ui_z | (exp_z << (27 - shift)) | ((frac64_z >> (5 + shift)) as u32);
            //Check if rounding bits in regime or exp and clean off unwanted bits
            if ((0x80000000_u32 >> N) & u_a) != 0 {
                if (((0x80000000_u32 >> (N - 1)) & u_a) != 0)
                    || (((0x7FFFFFFF_u32 >> N) & u_a) != 0)
                {
                    u_a = (u_a & (((-0x80000000_i32) >> (N - 1)) as u32))
                        + (0x80000000_u32 >> (N - 1));
                }
            }
            u_a
        };

        Self::from_bits(u_a & (((-0x80000000_i32) >> (N - 1)) as u32))
    }
}
