# DCB Parser - Lazy Record Loading

The DCB (DataCore Binary) parser now supports lazy record loading for efficient memory usage and fast initial load times.

## Overview

DCB files can contain hundreds of thousands of records with complex nested data. Loading all records eagerly can consume significant memory and time. Lazy loading solves this by:

1. **Loading metadata only** - Record headers, struct definitions, and indices are loaded immediately
2. **On-demand value loading** - Record property values are loaded only when accessed
3. **Automatic caching** - Once loaded, values are cached for fast subsequent access
4. **Memory management** - Individual records or all records can be unloaded to free memory

## API Usage

### Basic Lazy Loading

```rust
use starbreaker_parsers::dcb::DcbParser;
use std::path::Path;

// Create parser
let parser = DcbParser::new();

// Parse with lazy loading
let lazy_db = parser.parse_lazy(Path::new("data.dcb"))?;

// At this point, only metadata is in memory
println!("Total records: {}", lazy_db.record_count());
```

### Accessing Records

```rust
// Find records by struct type (still lazy)
let ships = lazy_db.find_by_struct("Ship");

for ship in ships {
    // Metadata is always available
    println!("Ship: {} (GUID: {:016X})", ship.name, ship.guid);
    
    // Load values on-demand
    let values = lazy_db.load_record(ship)?;
    
    // Access specific properties
    if let Some(mass) = values.get("mass") {
        println!("  Mass: {:?}", mass);
    }
}
```

### Memory Management

```rust
// Check if a record is loaded
if record.is_loaded() {
    println!("Record values are cached in memory");
}

// Unload a specific record to free memory
record.unload();

// Unload all cached values
lazy_db.unload_all();
```

### Converting to Eager Loading

```rust
// Convert to fully-loaded DataCore
// This loads all record values into memory
let eager_db = lazy_db.to_eager()?;

// Now all records are fully loaded
for record in &eager_db.records {
    // Direct access to all values
    println!("{}: {:?}", record.name, record.values);
}
```

## When to Use Lazy Loading

### Use Lazy Loading When:

- **Large DCB files** (>100k records)
- **Browsing/searching** - Only viewing metadata
- **Selective access** - Only need a subset of records
- **Memory constrained** - Limited available RAM
- **Fast startup** - Need quick initial load

### Use Eager Loading When:

- **Small DCB files** (<10k records)
- **Full processing** - Need to access all records
- **Performance critical** - Avoid lazy load overhead
- **Simple access patterns** - Don't need on-demand loading

## Performance Characteristics

### Lazy Loading
- **Initial Load**: Very fast (only metadata)
- **Memory**: Minimal (grows as records are accessed)
- **First Access**: Slight overhead (needs file I/O)
- **Subsequent Access**: Fast (cached in memory)

### Eager Loading
- **Initial Load**: Slower (loads everything)
- **Memory**: High (all data in memory)
- **First Access**: Fast (already loaded)
- **Subsequent Access**: Fast (all in memory)

## Implementation Details

### Record Storage

Each `LazyRecord` stores:
- Metadata (ID, name, GUID, struct ID)
- File offset for lazy loading
- Optional cached values (loaded on demand)

### Thread Safety

- Uses `Arc<RwLock<>>` for safe concurrent access
- Multiple threads can read cached values
- Lazy loading is synchronized

### File Handles

- Maintains a shared file handle for lazy loading
- Automatically managed by `LazyDataCore`
- Thread-safe access via `Mutex`

## Examples

See `examples/lazy_dcb_loading.rs` for a complete working example.

## Migration Guide

### From Eager to Lazy

```rust
// Old: Eager loading
let db = parser.parse_file(path)?;

// New: Lazy loading
let lazy_db = parser.parse_lazy(path)?;

// Access records (with lazy loading)
if let Some(record) = lazy_db.get_record(guid) {
    let values = lazy_db.load_record(record)?;
}

// Or convert to eager if needed
let db = lazy_db.to_eager()?;
```

## Best Practices

1. **Use lazy loading by default** for large files
2. **Unload records** you no longer need
3. **Batch operations** to minimize file I/O
4. **Cache frequently accessed** records
5. **Convert to eager** if you need to access everything

## Future Improvements

Potential enhancements:
- [ ] Configurable cache size limits
- [ ] LRU eviction policy
- [ ] Background preloading
- [ ] Compression for cached values
- [ ] Statistics/metrics tracking
