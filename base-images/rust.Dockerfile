# Self-contained Rust nightly base image
FROM scratch AS rootfs
ADD alpine-minirootfs-3.18.4-x86_64.tar.gz /

FROM scratch
COPY --from=rootfs / /

RUN apk add --no-cache gcc musl-dev curl ca-certificates pkgconfig openssl-dev openssl-libs-static make && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly && \
    source ~/.cargo/env && \
    rustup component add rust-src && \
    rm -rf /tmp/* /var/cache/apk/*

ENV PATH="/root/.cargo/bin:${PATH}" \
    RUSTUP_HOME="/root/.rustup" \
    CARGO_HOME="/root/.cargo"

CMD ["cargo", "--version"]
