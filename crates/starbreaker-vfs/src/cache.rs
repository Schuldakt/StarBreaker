use lru::LruCache;
use parking_lot::Mutex;

pub struct DecompressionCache {
    cache: Mutex<LruCache<PathBuf, Arc<Vec<u8>>>>,
    max_size_bytes: usize,
    current_size: AtomicUsize,
}