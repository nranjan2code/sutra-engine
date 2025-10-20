# Self-contained minimal runtime base
FROM scratch AS rootfs
ADD alpine-minirootfs-3.18.4-x86_64.tar.gz /

FROM scratch
COPY --from=rootfs / /

RUN apk add --no-cache \
    ca-certificates \
    curl \
    netcat-openbsd \
    bash \
    coreutils \
    procps \
    libssl3 \
    && rm -rf /var/cache/apk/* /tmp/*

ENV PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

CMD ["/bin/sh"]