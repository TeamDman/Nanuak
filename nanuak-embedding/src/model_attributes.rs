use crate::attributes::Accuracy;
use crate::attributes::ContextSize;
use crate::attributes::Latency;
use crate::attributes::Throughput;
use crate::attributes::VramRequirement;

pub struct ModelAttributes {
    pub vram_requirement: Option<VramRequirement>,
    pub latency: Option<Latency>,
    pub accuracy: Option<Accuracy>,
    pub throughput: Option<Throughput>,
    pub context_size: ContextSize,
}
