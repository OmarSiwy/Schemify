//! Line-record codec for every Schemify text file (D9): documents, project manifest,
//! lockfile, user config and the edit journal.
//!
//! Owns no tables. Parses bytes into records and attributes, writes the canonical form,
//! and merges/diffs at the record level (by kind + id, or kind + name).
//! Spec: `docs/spec/format.md` §2–§5.
