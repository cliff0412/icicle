#[cfg(feature = "arkworks")]
use ark_bw6_761::{g1::Config as ArkG1Config, Fq, Fr};
use icicle_core::curve::{Affine, Curve, Projective};
use icicle_core::field::{Field, MontgomeryConvertibleField};
use icicle_core::traits::{FieldConfig, FieldImpl, GenerateRandom};
use icicle_core::{impl_curve, impl_field, impl_scalar_field};
use icicle_cuda_runtime::device_context::{get_default_device_context, DeviceContext};
use icicle_cuda_runtime::error::CudaError;
use icicle_cuda_runtime::memory::HostOrDeviceSlice;

pub(crate) const SCALAR_LIMBS: usize = 6;
pub(crate) const BASE_LIMBS: usize = 12;

impl_scalar_field!("bw6_761", SCALAR_LIMBS, ScalarField, ScalarCfg, Fr);
impl_field!(BASE_LIMBS, BaseField, BaseCfg, Fq);
impl_curve!("bw6_761", CurveCfg, ScalarField, BaseField);

#[cfg(test)]
mod tests {
    use super::ScalarField;
    use super::{CurveCfg, BASE_LIMBS};
    use icicle_core::curve::Curve;
    use icicle_core::impl_curve_tests;
    use icicle_core::impl_field_tests;
    use icicle_core::tests::*;
    use icicle_core::traits::FieldImpl;

    impl_field_tests!(ScalarField);
    impl_curve_tests!(BASE_LIMBS, CurveCfg);
}
