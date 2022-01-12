
FROM mverleg/rust_nightly_musl_base:deps_2022-01-01_5

# Copy the actual code.
# exclude .lock file for now as it slows down dependencies
COPY ./Cargo.toml ./Cargo.lock ./build.rs ./deny.toml ./
COPY ./src ./src
COPY ./examples ./examples
COPY ./grammar ./grammar

# Build (for test)
RUN find . -name target -prune -o -type f &&\
    touch -c build.rs src/main.rs src/lib.rs &&\
    cargo build --bin pastad --all-features --tests

# Test
RUN cargo --offline test --all-features

# Lint
RUN cargo --offline clippy --all-features --tests -- -D warnings

# Style
RUN cargo --offline fmt --all -- --check

