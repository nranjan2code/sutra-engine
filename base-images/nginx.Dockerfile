# Self-contained Nginx base image
FROM scratch AS rootfs
ADD alpine-minirootfs-3.18.4-x86_64.tar.gz /

FROM scratch
COPY --from=rootfs / /

RUN apk add --no-cache nginx curl ca-certificates && \
    mkdir -p /var/cache/nginx /var/log/nginx /var/run && \
    rm -rf /var/cache/apk/* /tmp/*

# Forward request and error logs to docker log collector
RUN ln -sf /dev/stdout /var/log/nginx/access.log \
    && ln -sf /dev/stderr /var/log/nginx/error.log

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]