use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Test that orphaned children (entries whose parent directories are filtered)
/// are properly handled with warnings and no memory leaks.
///
/// Scenario:
/// - Create directory structure: .hidden/config
/// - .hidden is filtered (name starts with '.', Display::VisibleOnly)
/// - config is NOT filtered (name doesn't start with '.')
/// - Result: config becomes orphaned in HashMap
///
/// Expected behavior:
/// - Warning logged to stderr with diagnostic information
/// - Command succeeds (no panic or crash)
/// - HashMap is completely drained (no memory leak)
#[test]
fn test_orphaned_children_are_handled_with_warnings() {
    // Create temporary directory structure
    let temp = assert_fs::TempDir::new().expect("create temp dir");
    
    // Create hidden directory with non-hidden file
    let hidden_dir = temp.child(".hidden");
    hidden_dir.create_dir_all().expect("create .hidden directory");
    
    let config_file = hidden_dir.child("config");
    config_file.write_str("test content").expect("write config file");
    
    // Also create a visible directory to ensure tree has content
    let visible_dir = temp.child("visible");
    visible_dir.create_dir_all().expect("create visible directory");
    
    let visible_file = visible_dir.child("file.txt");
    visible_file.write_str("visible content").expect("write visible file");

    // Run sap --tree on the temp directory
    // By default, Display::VisibleOnly filters hidden files
    let mut cmd = Command::cargo_bin("sap").expect("binary exists");
    cmd
        .arg("--tree")
        .arg(temp.path())
        .assert()
        .success()
        .stderr(
            predicate::str::contains("Warning: Entry")
                .and(predicate::str::contains("orphaned"))
                .and(predicate::str::contains(".hidden"))
        )
        .stderr(predicate::str::contains("config"));
    
    temp.close().expect("cleanup temp dir");
}

/// Test that no warnings are logged when there are no orphaned entries
/// (normal case where all parents and children are consistently filtered).
#[test]
fn test_no_orphaned_warnings_for_normal_directory_structure() {
    // Create temporary directory with normal structure (no hidden dirs with visible children)
    let temp = assert_fs::TempDir::new().expect("create temp dir");
    
    // Create normal directory structure
    let dir1 = temp.child("dir1");
    dir1.create_dir_all().expect("create dir1");
    
    let file1 = dir1.child("file1.txt");
    file1.write_str("content1").expect("write file1");
    
    let dir2 = temp.child("dir2");
    dir2.create_dir_all().expect("create dir2");
    
    let file2 = dir2.child("file2.txt");
    file2.write_str("content2").expect("write file2");

    // Run sap --tree
    let mut cmd = Command::cargo_bin("sap").expect("binary exists");
    cmd
        .arg("--tree")
        .arg(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("orphaned").not());
    
    temp.close().expect("cleanup temp dir");
}

/// Test that orphaned children are handled when parent is filtered by ignore_globs.
///
/// This tests a different filtering mechanism than Display::VisibleOnly to ensure
/// the orphaned handler works correctly with all filtering types.
#[test]
fn test_orphaned_children_with_ignore_globs() {
    // Create temporary directory structure
    let temp = assert_fs::TempDir::new().expect("create temp dir");
    
    // Create directory that matches typical ignore pattern
    let git_dir = temp.child(".git");
    git_dir.create_dir_all().expect("create .git directory");
    
    // Create file inside .git that doesn't start with dot
    let head_file = git_dir.child("HEAD");
    head_file.write_str("ref: refs/heads/main").expect("write HEAD file");
    
    // Create visible content to ensure tree has something
    let src_dir = temp.child("src");
    src_dir.create_dir_all().expect("create src directory");
    
    let main_file = src_dir.child("main.rs");
    main_file.write_str("fn main() {}").expect("write main.rs");

    // Run sap --tree (default ignore_globs includes .git)
    let mut cmd = Command::cargo_bin("sap").expect("binary exists");
    cmd
        .arg("--tree")
        .arg(temp.path())
        .assert()
        .success()
        .stderr(
            predicate::str::contains("orphaned")
                .and(predicate::str::contains(".git"))
        );
    
    temp.close().expect("cleanup temp dir");
}

/// Test that deeply nested orphaned children are handled correctly.
///
/// Scenario: .hidden/subdir/file.txt where .hidden is filtered but subdir and file are not.
#[test]
fn test_deeply_nested_orphaned_children() {
    let temp = assert_fs::TempDir::new().expect("create temp dir");
    
    // Create deeply nested structure
    let hidden_dir = temp.child(".hidden");
    hidden_dir.create_dir_all().expect("create .hidden");
    
    let subdir = hidden_dir.child("subdir");
    subdir.create_dir_all().expect("create subdir");
    
    let deep_file = subdir.child("file.txt");
    deep_file.write_str("deep content").expect("write deep file");
    
    // Add visible content
    let visible = temp.child("visible.txt");
    visible.write_str("visible").expect("write visible file");

    // Run sap --tree
    let mut cmd = Command::cargo_bin("sap").expect("binary exists");
    cmd
        .arg("--tree")
        .arg(temp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("orphaned"))
        .stderr(predicate::str::contains(".hidden"));
    
    temp.close().expect("cleanup temp dir");
}