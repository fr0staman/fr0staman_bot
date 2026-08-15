#!/usr/bin/env bash
#
# Applies the migrations to the database the integration tests use.
#
#   createdb fr0staman_test
#   export TEST_DATABASE_URL=postgres://user:pass@localhost/fr0staman_test
#   ./scripts/setup_test_db.sh
#
# `diesel_migrations` is not a dependency (it is sync-only, and several
# migrations use CREATE INDEX CONCURRENTLY with run_in_transaction = false),
# so the CLI is the supported way to provision the schema.

set -euo pipefail

if [[ -z "${TEST_DATABASE_URL:-}" ]]; then
    echo "TEST_DATABASE_URL is not set." >&2
    echo "  export TEST_DATABASE_URL=postgres://user:pass@localhost/fr0staman_test" >&2
    exit 1
fi

if ! command -v diesel >/dev/null 2>&1; then
    echo "diesel CLI not found. Install it with:" >&2
    echo "  cargo install diesel_cli --no-default-features --features=postgres" >&2
    exit 1
fi

cd "$(dirname "$0")/.."

echo "Applying migrations to ${TEST_DATABASE_URL%%\?*}"
diesel migration run --database-url "$TEST_DATABASE_URL"
echo "Done. Run the suite with: cargo test"
