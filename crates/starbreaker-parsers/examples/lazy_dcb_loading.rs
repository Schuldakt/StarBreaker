/// Example demonstrating lazy record loading for DCB files
/// 
/// This example shows how to use the LazyDataCore to load DCB files
/// with minimal memory footprint by only loading record values on-demand.

use starbreaker_parsers::dcb::DcbParser;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create parser
    let parser = DcbParser::new();
    
    // Example DCB file path (replace with actual path)
    let dcb_path = Path::new("path/to/file.dcb");
    
    if !dcb_path.exists() {
        println!("Example DCB file not found. This is a demonstration of the API.");
        println!("Usage: cargo run --example lazy_dcb_loading -- <path-to-dcb-file>");
        return Ok(());
    }
    
    println!("Loading DCB file with lazy loading...");
    
    // Parse with lazy loading - only metadata is loaded
    let lazy_db = parser.parse_lazy(dcb_path)?;
    
    println!("✓ Loaded {} records (metadata only)", lazy_db.record_count());
    println!("✓ Found {} struct types", lazy_db.structs.len());
    println!("✓ Memory usage: minimal (only metadata loaded)");
    println!();
    
    // List all struct types
    println!("Available struct types:");
    for name in lazy_db.struct_names() {
        let count = lazy_db.find_by_struct(name).len();
        println!("  - {}: {} records", name, count);
    }
    println!();
    
    // Find records of a specific type (still lazy)
    if let Some(struct_name) = lazy_db.struct_names().first() {
        println!("Examining {} records:", struct_name);
        let records = lazy_db.find_by_struct(struct_name);
        
        for (i, lazy_record) in records.iter().take(5).enumerate() {
            println!("  {}. {} (GUID: {:016X})", 
                i + 1, 
                lazy_record.name,
                lazy_record.guid
            );
            
            // Values are not loaded yet
            println!("     Loaded: {}", lazy_record.is_loaded());
            
            // Load values for this specific record
            let values = lazy_db.load_record(lazy_record)?;
            println!("     Properties: {}", values.len());
            
            // Now it's loaded
            println!("     Loaded: {}", lazy_record.is_loaded());
            
            // Can unload to free memory
            lazy_record.unload();
            println!("     After unload: {}", lazy_record.is_loaded());
        }
    }
    println!();
    
    // Working with specific records
    println!("Lazy record lookup:");
    if let Some(first_record) = lazy_db.records.first() {
        println!("  Record ID: {}", first_record.id);
        println!("  Name: {}", first_record.name);
        println!("  GUID: {:016X}", first_record.guid);
        
        // Load values on-demand
        let values = lazy_db.load_record(first_record)?;
        println!("  Loaded {} properties", values.len());
    }
    println!();
    
    // Memory management
    println!("Memory management:");
    println!("  - Load specific records as needed");
    println!("  - Unload individual records: record.unload()");
    println!("  - Unload all records: lazy_db.unload_all()");
    
    // Unload all cached values
    lazy_db.unload_all();
    println!("  ✓ All records unloaded");
    println!();
    
    // Converting to eager loading
    println!("Converting to fully-loaded DataCore:");
    println!("  This loads all record values into memory");
    
    // Uncomment to convert to eager (loads everything)
    // let eager_db = lazy_db.to_eager()?;
    // println!("  ✓ Loaded {} records with full data", eager_db.record_count());
    
    println!();
    println!("Lazy loading benefits:");
    println!("  ✓ Fast initial load (only metadata)");
    println!("  ✓ Low memory usage (load on demand)");
    println!("  ✓ Good for large DCB files");
    println!("  ✓ Ideal for browsing/searching");
    
    Ok(())
}
