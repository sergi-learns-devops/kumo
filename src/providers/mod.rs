pub mod aws;
pub mod azure;
pub mod gcp;

use crate::models::InstanceType;

#[allow(dead_code)]
pub trait ProviderCatalog {
    fn catalog() -> &'static [InstanceType];
    fn supports_custom() -> bool;
    fn valid_regions() -> &'static [&'static str];
}
