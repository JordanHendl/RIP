extern crate unidiff;

use unidiff::{Hunk, PatchedFile};

#[test]
fn test_is_added_file() {
    let hunk = Hunk::new(0, 0, 0, 1, "");
    let file = PatchedFile::with_hunks("a", "b", vec![hunk]);
    assert!(file.is_added_file());
}

#[test]
fn test_is_removed_file() {
    let hunk = Hunk::new(0, 1, 0, 0, "");
    let file = PatchedFile::with_hunks("a", "b", vec![hunk]);
    assert!(file.is_removed_file());
}

#[test]
fn test_is_modified_file() {
    let hunk = Hunk::new(0, 1, 0, 1, "");
    let file = PatchedFile::with_hunks("a", "b", vec![hunk]);
    assert!(file.is_modified_file());
}
