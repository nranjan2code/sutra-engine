# Self-contained Node.js 18 base image
FROM scratch AS rootfs
ADD alpine-minirootfs-3.18.4-x86_64.tar.gz /

FROM scratch
COPY --from=rootfs / /

RUN apk add --no-cache \
    nodejs=~18 \
    npm \
    yarn \
    curl \
    ca-certificates \
    && rm -rf /var/cache/apk/* /tmp/*

ENV NODE_ENV=production \
    NODE_PATH=/usr/lib/node_modules

CMD ["node", "--version"]