pub struct P4kDiff {
    pub added: Vec<P4kEntry>,
    pub removed: Vec<P4kEntry>,
    pub modified: Vec<(P4kEntry, P4kEntry)>, // (old, new)
}

impl P4kDiff {
    pub fn compare(old: &P4kArchive, new: &P4kArchive) -> Self { /* ... */ }
    pub fn export_report(&self, format: ReportFormat) -> String { /* ... */ }
}