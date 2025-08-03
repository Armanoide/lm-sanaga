serve:
    cargo run --bin lm-sanaga
cli *args:
    cargo run --bin cli-sanaga -- {{args}}

hf_benchmark_inference:
    cargo build --release --bin benchmark_inference
    hyperfine --runs 10 \
        "target/release/benchmark_inference"

dhat_benchmark_inference:
    cargo build --release --bin benchmark_inference
    cargo run --release --bin benchmark_inference --features dhat-heap

samply_benchmark_inference:
    cargo build --release --bin benchmark_inference
    samply record --output benchmark_inference.samply \
        "target/release/benchmark_inference"


db_migration:
    sea-orm-cli migrate up \
    --migration-dir ./sn_backend/src/db/migration \

db_migration_revert:
    sea-orm-cli migrate down \
    --migration-dir ./sn_backend/src/db/migration \

