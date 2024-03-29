use ark_ec::CurveConfig;
use ark_ed25519::{EdwardsConfig, EdwardsProjective};

pub type Zp = <EdwardsConfig as CurveConfig>::ScalarField;
pub type G = EdwardsProjective;

pub trait IsZero {
    fn is_zero(&self) -> bool;
}
