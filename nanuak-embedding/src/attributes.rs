use uom::si::f32::Information;
use uom::si::f32::InformationRate;
use uom::si::f32::Ratio;
use uom::si::f32::Time;

#[derive(Debug, Eq, PartialEq)]
pub enum Residency {
    Local,
    RemoteSameCountry,
    RemoteAnywhere,
}
pub struct VramRequirement(pub Information);

pub struct Latency(pub Time);
pub struct Accuracy(pub Ratio);
pub struct Throughput(pub InformationRate);
pub struct ContextSize(pub u32);
