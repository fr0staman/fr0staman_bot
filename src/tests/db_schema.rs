//! Migration drift: a migration landing without regenerating
//! `src/db/schema.rs` keeps compiling and fails at runtime instead.
//! Needs `TEST_DATABASE_URL` and the `diesel` CLI; skips otherwise.

use std::path::PathBuf;
use std::process::Command;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_checked_in_schema_matches_the_migrated_database() {
    let _ = dotenvy::from_path(manifest_dir().join(".env"));

    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipping: TEST_DATABASE_URL is not set");
        return;
    };

    let output = match Command::new("diesel")
        .current_dir(manifest_dir())
        .args(["print-schema", "--database-url", &url])
        .output()
    {
        Ok(output) => output,
        Err(_) => {
            eprintln!("skipping: the diesel CLI is not installed");
            return;
        },
    };

    assert!(
        output.status.success(),
        "diesel print-schema failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = String::from_utf8_lossy(&output.stdout);
    let checked_in =
        std::fs::read_to_string(manifest_dir().join("src/db/schema.rs"))
            .expect("src/db/schema.rs is missing");

    // Ignore blank lines and trailing whitespace: rustfmt and the generator
    // disagree on those.
    let normalize = |s: &str| {
        s.lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };

    assert_eq!(
        normalize(&checked_in),
        normalize(&generated),
        "src/db/schema.rs is out of date — run `diesel print-schema > \
         src/db/schema.rs` after adding a migration"
    );
}
