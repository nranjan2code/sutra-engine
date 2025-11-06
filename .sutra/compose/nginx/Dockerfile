FROM nginx:1.25-alpine

# Install required packages
RUN apk add --no-cache \
    openssl \
    ca-certificates

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Create directories for SSL certificates and logs
RUN mkdir -p /etc/nginx/ssl /var/log/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chmod 755 /etc/nginx/ssl

# Generate self-signed certificate for development (will be replaced in production)
RUN openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /etc/nginx/ssl/key.pem \
    -out /etc/nginx/ssl/cert.pem \
    -subj "/C=US/ST=Development/L=Development/O=Sutra/CN=localhost"

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:80/health || exit 1

EXPOSE 80 443 8080

CMD ["nginx", "-g", "daemon off;"]
