macro_rules! bitflag {
    (
        $(#[doc = $outer_doc:literal])*
        enum $name:ident {
            $(
                $(#[doc = $doc:literal])*
                $flag:ident = $shift:literal
            ),+
            $(,)?
        }
    ) => {
        #[autodoc(category = "Flags")]
        $(#[doc = $outer_doc])*
        #[doc = "## Bits"]
        #[doc = "|Flag|Bit|Description|\n|-|-|-|"]
        $(
            $(#[doc = $doc])*
        )+
        #[derive(Debug, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        pub struct $name{ inner: u128 }

        impl $name {
            pub fn empty() -> Self {
                Self { inner: 0 }
            }

            pub fn all() -> Self {
                Self { inner: $((1 << $shift) |)+ 0 }
            }

            pub fn from_bits(bits: u128) -> Self {
                Self { inner: bits }
            }

            pub fn bits(&self) -> u128 {
                self.inner
            }

        $(
            #[allow(non_snake_case)]
            pub fn $flag(&self) -> bool {
                (self.inner & (1 << $shift)) == (1 << $shift)
            }
        )+
        }

        impl std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output { Self {inner: self.inner & rhs.inner} }
        }

        impl std::ops::BitAnd<u128> for $name {
            type Output = Self;
            fn bitand(self, rhs: u128) -> Self::Output { Self {inner: self.inner & rhs} }
        }

        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) { self.inner &= rhs.inner; }
        }

        impl std::ops::BitAndAssign<u128> for $name {
            fn bitand_assign(&mut self, rhs: u128) { self.inner &= rhs; }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output { Self {inner: self.inner | rhs.inner} }
        }

        impl std::ops::BitOr<u128> for $name {
            type Output = Self;
            fn bitor(self, rhs: u128) -> Self::Output { Self {inner: self.inner | rhs} }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.inner |= rhs.inner;
            }
        }

        impl std::ops::BitOrAssign<u128> for $name {
            fn bitor_assign(&mut self, rhs: u128) {
                self.inner |= rhs;
            }
        }

        impl std::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self { inner: self.inner ^ rhs.inner }
            }
        }

        impl std::ops::BitXor<u128> for $name {
            type Output = Self;
            fn bitxor(self, rhs: u128) -> Self::Output {
                Self { inner: self.inner ^ rhs }
            }
        }

        impl std::ops::BitXorAssign for $name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.inner ^= rhs.inner;
            }
        }

        impl std::ops::BitXorAssign<u128> for $name {
            fn bitxor_assign(&mut self, rhs: u128) {
                self.inner ^= rhs;
            }
        }

        impl std::ops::Neg for $name {
            type Output = Self;
            fn neg(self) -> Self::Output {
                Self { inner: !self.inner }
            }
        }

        impl std::ops::Shl<u32> for $name {
            type Output = Self;
            fn shl(self, rhs: u32) -> Self::Output {
                Self { inner: self.inner << rhs }
            }
        }

        impl std::ops::ShlAssign<u32> for $name {
            fn shl_assign(&mut self, rhs: u32) {
                self.inner <<= rhs;
            }
        }

        impl std::ops::Shr<u32> for $name {
            type Output = Self;
            fn shr(self, rhs: u32) -> Self::Output {
                Self { inner: self.inner >> rhs }
            }
        }

        impl std::ops::ShrAssign<u32> for $name {
            fn shr_assign(&mut self, rhs: u32) {
                self.inner >>= rhs;
            }
        }
    }
}
