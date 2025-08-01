mod cache;
mod segmented;

pub use cache::Cache;
pub use cache::DefaultLifecycle;
pub use cache::Lifecycle;
pub use cache::One;
pub use cache::Weigher;
pub use segmented::SegmentedCache;
