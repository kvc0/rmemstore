use bytes::Bytes;

use super::MemstoreItem;

#[derive(Clone, Debug)]
pub struct MemstoreWeigher;
impl k_cache::Weigher<Bytes, MemstoreItem> for MemstoreWeigher {
    fn weigh(key: &Bytes, item: &MemstoreItem) -> usize {
        MemstoreItem::weigher(key, item) as usize
    }
}
