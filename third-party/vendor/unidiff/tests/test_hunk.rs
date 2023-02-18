extern crate unidiff;

use unidiff::{Hunk, Line};

#[test]
fn test_default_is_valid() {
    let hunk = Hunk::new(0, 0, 0, 0, "");
    assert!(hunk.is_valid());
}

#[test]
fn test_missing_data_is_not_valid() {
    let hunk = Hunk::new(0, 1, 0, 1, "");
    assert!(!hunk.is_valid());
}

#[test]
fn test_append_context() {
    let mut hunk = Hunk::new(0, 1, 0, 1, "");
    hunk.append(Line::new("sample line", " "));
    assert!(hunk.is_valid());
    assert_eq!(hunk.source_lines(), hunk.target_lines());
}

#[test]
fn test_append_added_line() {
    let mut hunk = Hunk::new(0, 0, 0, 1, "");
    hunk.append(Line::new("sample line", "+"));
    assert!(hunk.is_valid());
    assert_eq!(0, hunk.source_lines().len());
    assert_eq!(1, hunk.target_lines().len());
}

#[test]
fn test_append_removed_line() {
    let mut hunk = Hunk::new(0, 1, 0, 0, "");
    hunk.append(Line::new("sample line", "-"));
    assert!(hunk.is_valid());
    assert_eq!(1, hunk.source_lines().len());
    assert_eq!(0, hunk.target_lines().len());
}
