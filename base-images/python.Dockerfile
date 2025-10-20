# Self-contained Python 3.11 base image
FROM scratch AS rootfs
ADD alpine-minirootfs-3.18.4-x86_64.tar.gz /

FROM scratch
COPY --from=rootfs / /

RUN apk add --no-cache python3 python3-dev py3-pip gcc g++ musl-dev libffi-dev openssl-dev curl ca-certificates linux-headers libc6-compat && \
    ln -sf python3 /usr/bin/python && \
    python -m ensurepip && \
    pip3 install --no-cache-dir --upgrade pip setuptools wheel typing_extensions msgpack && \
    rm -rf /root/.cache /tmp/* /var/cache/apk/*

ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1

CMD ["python3"]