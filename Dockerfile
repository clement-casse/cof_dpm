FROM rust:1.92 AS build-env

USER root

RUN set -eux; { \
    apt-get update; \
    apt-get install --no-install-recommends -y \
        pkg-config \
        libprotobuf-dev \
        openssl \
        protobuf-compiler \
    ; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*; \
}

RUN set -eux; { \
    cargo install --locked cargo-chef sccache; \
}

ENV RUSTC_WRAPPER=sccache \
    SCCACHE_DIR=/sccache

WORKDIR /usr/src/cof_dpm

FROM build-env AS planner

COPY Cargo.toml Cargo.lock ./
COPY aio_server/Cargo.toml ./aio_server/Cargo.toml
RUN mkdir ./aio_server/src && echo 'fn main() {}' > ./aio_server/src/main.rs
COPY dice_service/Cargo.toml ./dice_service/Cargo.toml
RUN mkdir ./dice_service/src && touch ./dice_service/src/lib.rs
COPY user_service/Cargo.toml ./user_service/Cargo.toml
RUN mkdir ./user_service/src && touch ./user_service/src/lib.rs

RUN set -eux; { \
    cargo chef prepare --recipe-path recipe.json; \
}

FROM build-env AS builder

COPY --from=planner /usr/src/cof_dpm/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    set -eux; { \
        cargo chef cook --release --recipe-path recipe.json; \
    }

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    set -eux; { \
        cargo build --release --package aio_server; \
    }

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /usr/src/cof_dpm/target/release/aio_server /
ENTRYPOINT [ "/aio_server" ]
