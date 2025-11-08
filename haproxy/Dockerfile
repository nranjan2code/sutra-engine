FROM haproxy:2.9-alpine

# Copy configuration
COPY ml-base-lb.cfg /usr/local/etc/haproxy/haproxy.cfg

# Note: Skipping validation at build time (DNS names not available)
# Configuration will be validated at runtime when containers start

# Expose ports
EXPOSE 8887 9999

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD haproxy -c -f /usr/local/etc/haproxy/haproxy.cfg || exit 1

CMD ["haproxy", "-f", "/usr/local/etc/haproxy/haproxy.cfg"]
