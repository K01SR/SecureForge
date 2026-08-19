//! NTFS metadata scrubber.
//!
//! Targets:
//! - MFT (Master File Table) record for the file
//!   - $STANDARD_INFORMATION attribute
//!   - $FILE_NAME attribute(s)
//!   - $DATA attribute (resident data for small files)
//!   - Run lists for non-resident data
//! - $LogFile journal references
//! - $UsnJrnl (Update Sequence Number Journal) entries
