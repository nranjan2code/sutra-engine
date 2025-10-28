# Sutra Client (Web UI) - Simple Build
FROM node:20-alpine AS builder

RUN apk add --no-cache python3 make g++
WORKDIR /app

COPY packages/sutra-client/package*.json ./
COPY packages/sutra-client/tsconfig*.json ./
COPY packages/sutra-client/vite.config.ts ./

RUN npm ci --silent && \
    npm cache clean --force && \
    rm -rf ~/.npm

COPY packages/sutra-client/index.html ./
COPY packages/sutra-client/src/ ./src/
RUN npm run build && \
    find dist -name "*.map" -delete

# Final stage
FROM nginx:1.26-alpine

COPY --from=builder /app/dist /usr/share/nginx/html

# Simple nginx configuration
RUN cat > /etc/nginx/nginx.conf << 'EOF'
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /tmp/nginx.pid;

events {
    worker_connections 1024;
    use epoll;
    multi_accept on;
}

http {
    client_body_temp_path /tmp/client_temp;
    proxy_temp_path       /tmp/proxy_temp_path;
    fastcgi_temp_path     /tmp/fastcgi_temp;
    uwsgi_temp_path       /tmp/uwsgi_temp;
    scgi_temp_path        /tmp/scgi_temp;
    
    include /etc/nginx/mime.types;
    default_type application/octet-stream;
    
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;
    
    gzip on;
    gzip_vary on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml text/javascript;
    
    server {
        listen 8080;
        server_name localhost;
        root /usr/share/nginx/html;
        index index.html;
        
        location / {
            try_files $uri $uri/ /index.html;
        }
        
        location /api/ {
            proxy_pass http://sutra-api:8000/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
EOF

RUN addgroup -g 1000 app && \
    adduser -D -s /bin/sh -u 1000 -G app app && \
    chown -R app:app /usr/share/nginx/html && \
    chmod -R 755 /usr/share/nginx/html && \
    touch /var/run/nginx.pid && \
    chown -R app:app /var/cache/nginx /var/run /var/log/nginx

USER app
EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]