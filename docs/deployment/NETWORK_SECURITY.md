# Network Security Architecture

**Version:** 1.0
**Date:** 2025-01-06
**Status:** Production Ready

---

## Overview

This document describes the network security architecture for Sutra AI deployment, including the nginx reverse proxy configuration, service isolation, and network exposure policies.

### Key Security Principles

1. **Single Entry Point**: All external traffic routes through nginx reverse proxy
2. **Internal-Only Services**: Storage, ML, and infrastructure services NOT exposed externally
3. **Defense in Depth**: Multiple layers of security (TLS, authentication, rate limiting, network isolation)
4. **Principle of Least Privilege**: Only expose what's necessary for operation

---

## Architecture Diagram

```
                                    Internet
                                       │
                                       ▼
                           ┌────────────────────┐
                           │   Nginx Proxy      │
                           │   (80, 443, 8080)  │
                           └────────┬───────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
           ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
           │  sutra-api  │  │sutra-hybrid │  │sutra-client │
           │  (8000)     │  │  (8000)     │  │  (8080)     │
           └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
                  │                │                 │
                  └────────┬───────┴─────────────────┘
                           │ Internal Network Only
                  ┌────────┴───────────────────┐
                  │                            │
                  ▼                            ▼
         ┌─────────────────┐         ┌─────────────────┐
         │ storage-server  │         │ ml-base-service │
         │   (50051)       │         │    (8887)       │
         │  INTERNAL ONLY  │         │  INTERNAL ONLY  │
         └─────────────────┘         └─────────────────┘
```

---

## Network Exposure Policy

### ✅ EXPOSED Services (via Nginx Proxy Only)

These services are accessible from outside the Docker network, but ONLY through the nginx reverse proxy:

| Service | Internal Port | Nginx Route | Purpose | Auth Required |
|---------|---------------|-------------|---------|---------------|
| **nginx-proxy** | 80, 443, 8080 | / | Reverse proxy & load balancer | N/A |
| **sutra-api** | 8000 | /api/ | REST API endpoints | ✅ Yes |
| **sutra-hybrid** | 8000 | /sutra/ | Semantic + NLG API | ✅ Yes |
| **sutra-client** | 8080 | / | Main web UI | Via API |
| **sutra-control** | 9000 | /control/ | Control center UI | Via API |
| **sutra-explorer-backend** | 8100 | /explorer-api/ | Storage visualization API | ✅ Yes |
| **sutra-explorer-frontend** | 3000 | /explorer/ | Explorer web UI | Via API |
| **sutra-bulk-ingester** | 8005 | /ingest/ | Bulk data ingestion | ✅ Yes |

### ❌ INTERNAL ONLY Services (NOT Exposed)

These services are accessible ONLY within the Docker network:

| Service | Internal Port | Purpose | Accessed By |
|---------|---------------|---------|-------------|
| **storage-server** | 50051 | Core knowledge graph storage | sutra-api, sutra-hybrid |
| **grid-event-storage** | 50051 | Grid observability events | grid-master, grid-agents |
| **user-storage-server** | 50051 | User data & authentication | sutra-api |
| **grid-master** | 7001, 7002 | Grid orchestration (Enterprise) | grid-agents |
| **grid-agent-1/2** | 8001 | Node management (Enterprise) | grid-master |
| **ml-base-service** | 8887 | ML inference engine | embedding/nlg services |
| **embedding-single** | 8888 | Embedding service | storage-server, sutra-hybrid |
| **embedding-ha** | 8888, 8404 | HA embedding + HAProxy stats | storage-server, sutra-hybrid |
| **nlg-single** | 8003 | NLG service | sutra-hybrid |
| **nlg-ha** | 8889, 8405 | HA NLG + HAProxy stats | sutra-hybrid |

---

## Nginx Reverse Proxy Configuration

### Ports

- **Port 80** (HTTP): Redirects to HTTPS in production, allows health checks
- **Port 443** (HTTPS): Main production entry point with TLS 1.2/1.3
- **Port 8080** (HTTP): Development mode without SSL

### Security Features

#### 1. TLS/SSL Configuration

```nginx
# Modern TLS configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:...';
ssl_prefer_server_ciphers off;
```

**Production Setup:**
```bash
# Set environment variables for SSL certificates
export SUTRA_SSL_CERT_PATH=/path/to/your/cert.pem
export SUTRA_SSL_KEY_PATH=/path/to/your/key.pem

# Or use Let's Encrypt (recommended)
certbot certonly --standalone -d yourdomain.com
export SUTRA_SSL_CERT_PATH=/etc/letsencrypt/live/yourdomain.com/fullchain.pem
export SUTRA_SSL_KEY_PATH=/etc/letsencrypt/live/yourdomain.com/privkey.pem
```

#### 2. Rate Limiting

Multiple rate limit zones protect against abuse:

- **auth_limit**: 10 requests/minute for authentication endpoints
- **api_limit**: 60 requests/minute for API endpoints
- **general_limit**: 120 requests/minute for general traffic

```nginx
# Authentication endpoints (stricter)
location /api/auth {
    limit_req zone=auth_limit burst=5 nodelay;
    # ...
}

# API endpoints
location /api/ {
    limit_req zone=api_limit burst=10 nodelay;
    # ...
}
```

#### 3. Security Headers

All responses include comprehensive security headers:

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
add_header Permissions-Policy "geolocation=(), microphone=(), camera=()" always;
```

For production, enable HSTS:
```nginx
add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
```

#### 4. Internal Metrics Protection

Internal monitoring endpoints are restricted to internal networks only:

```nginx
location /internal/ {
    # Allow only from internal networks
    allow 10.0.0.0/8;
    allow 172.16.0.0/12;
    allow 192.168.0.0/16;
    allow 127.0.0.1;
    deny all;
    # ...
}
```

---

## Deployment Guide

### Quick Start (Development)

```bash
# 1. Build nginx proxy
cd .sutra/compose
docker build -t sutra-works-nginx-proxy:latest -f nginx/Dockerfile nginx/

# 2. Deploy with nginx proxy
export SUTRA_EDITION=simple  # or community, enterprise
docker-compose -f production.yml up -d

# 3. Verify nginx is running
docker ps | grep nginx-proxy
curl http://localhost:8080/health

# 4. Access services
# Main UI: http://localhost:8080/
# API: http://localhost:8080/api/
# Control: http://localhost:8080/control/
```

### Production Deployment

```bash
# 1. Generate/obtain SSL certificates
# Option A: Let's Encrypt
certbot certonly --standalone -d sutra.yourdomain.com

# Option B: Self-signed (development only)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/sutra-key.pem \
  -out /etc/ssl/certs/sutra-cert.pem

# 2. Set SSL certificate paths
export SUTRA_SSL_CERT_PATH=/etc/letsencrypt/live/sutra.yourdomain.com/fullchain.pem
export SUTRA_SSL_KEY_PATH=/etc/letsencrypt/live/sutra.yourdomain.com/privkey.pem

# 3. Enable security features
export SUTRA_SECURE_MODE=true
export SUTRA_EDITION=enterprise

# 4. Deploy
cd .sutra/compose
docker-compose -f production.yml up -d

# 5. Verify HTTPS
curl https://sutra.yourdomain.com/health

# 6. Check TLS configuration
openssl s_client -connect sutra.yourdomain.com:443 -tls1_3
```

### Network Verification

```bash
# Verify no services expose ports directly (except nginx)
docker ps --format "table {{.Names}}\t{{.Ports}}"

# Should only show sutra-works-nginx-proxy with ports 80, 443, 8080
# All other services should show no host ports

# Test internal-only service is NOT accessible
curl http://localhost:50051  # Should fail - connection refused

# Test service is accessible via nginx
curl http://localhost:8080/api/health  # Should succeed

# Validate naming conventions
./scripts/validate-naming.sh
```

---

## Security Best Practices

### ✅ DO

1. **Use HTTPS in Production**: Always deploy with valid SSL certificates
2. **Restrict Internal Endpoints**: Keep `/internal/` endpoints blocked from public access
3. **Enable HSTS**: Uncomment HSTS header in production nginx config
4. **Monitor Nginx Logs**: Regularly review access and error logs
5. **Update Rate Limits**: Adjust based on your traffic patterns
6. **Use Strong Secrets**: Set `SUTRA_AUTH_SECRET` to a strong random value
7. **Enable Authentication**: Use `SUTRA_SECURE_MODE=true` in production
8. **Regular Updates**: Keep nginx and base images updated

### ❌ DON'T

1. **Expose Storage Ports**: Never add `ports:` to storage-server or other internal services
2. **Disable Security Headers**: Keep all security headers enabled
3. **Use Self-Signed Certs in Prod**: Only use self-signed certificates for development
4. **Allow Public Internal Routes**: Never expose `/internal/` publicly
5. **Ignore Rate Limit Alerts**: High rate limit hits may indicate abuse
6. **Skip TLS Certificate Validation**: Always validate certificates in production
7. **Hardcode Secrets**: Use environment variables for all sensitive configuration

---

## Monitoring & Logging

### Nginx Access Logs

Access logs include detailed timing information:

```
$remote_addr - $remote_user [$time_local] "$request"
$status $body_bytes_sent "$http_referer"
"$http_user_agent" "$http_x_forwarded_for"
rt=$request_time uct="$upstream_connect_time"
uht="$upstream_header_time" urt="$upstream_response_time"
```

View logs:
```bash
# View nginx access logs
docker logs sutra-works-nginx-proxy

# View logs from volume
docker exec sutra-works-nginx-proxy cat /var/log/nginx/access.log

# Follow error logs
docker exec sutra-works-nginx-proxy tail -f /var/log/nginx/error.log
```

### Health Checks

```bash
# Nginx health
curl http://localhost:8080/health

# API health (via proxy)
curl http://localhost:8080/api/health

# Internal metrics (from inside network)
docker exec sutra-nginx-proxy curl http://sutra-api:8000/internal/health
```

### Rate Limit Monitoring

Monitor rate limit events:
```bash
# Check for rate limit errors
docker logs sutra-works-nginx-proxy 2>&1 | grep "limiting requests"

# View rate limit statistics
docker exec sutra-works-nginx-proxy cat /var/log/nginx/error.log | grep limit_req
```

---

## Firewall Configuration

### Recommended Firewall Rules

#### Ubuntu/Debian (ufw)

```bash
# Allow SSH (if needed)
ufw allow 22/tcp

# Allow HTTP/HTTPS only
ufw allow 80/tcp
ufw allow 443/tcp

# Allow development port (optional, development only)
ufw allow 8080/tcp

# Block all other ports
ufw default deny incoming
ufw default allow outgoing

# Enable firewall
ufw enable
```

#### RHEL/CentOS (firewalld)

```bash
# Allow HTTP/HTTPS
firewall-cmd --permanent --add-service=http
firewall-cmd --permanent --add-service=https

# Allow development port (optional)
firewall-cmd --permanent --add-port=8080/tcp

# Reload firewall
firewall-cmd --reload
```

#### Cloud Provider Security Groups

**AWS Security Group Rules:**
```
Type: HTTP (80)       Source: 0.0.0.0/0
Type: HTTPS (443)     Source: 0.0.0.0/0
Type: Custom (8080)   Source: YOUR_IP/32  (development only)
```

---

## Troubleshooting

### Service Not Accessible via Proxy

```bash
# 1. Check nginx is running
docker ps | grep sutra-works-nginx-proxy

# 2. Check nginx configuration is valid
docker exec sutra-works-nginx-proxy nginx -t

# 3. Check backend service is healthy
docker exec sutra-works-nginx-proxy curl http://sutra-works-api:8000/health

# 4. Check nginx logs for errors
docker logs sutra-works-nginx-proxy | grep error
```

### SSL Certificate Issues

```bash
# Verify certificate is mounted correctly
docker exec sutra-works-nginx-proxy ls -la /etc/nginx/ssl/

# Test certificate validity
docker exec sutra-works-nginx-proxy openssl x509 -in /etc/nginx/ssl/cert.pem -noout -dates

# Check certificate matches key
docker exec sutra-works-nginx-proxy openssl x509 -in /etc/nginx/ssl/cert.pem -noout -modulus | openssl md5
docker exec sutra-works-nginx-proxy openssl rsa -in /etc/nginx/ssl/key.pem -noout -modulus | openssl md5
# Hashes should match
```

### Rate Limiting Issues

```bash
# Check current rate limit configuration
docker exec sutra-works-nginx-proxy grep -A 5 "limit_req_zone" /etc/nginx/nginx.conf

# Temporarily disable rate limiting (testing only)
# Edit nginx.conf and comment out limit_req lines, then:
docker exec sutra-works-nginx-proxy nginx -s reload
```

### Internal Service Exposed by Mistake

If you accidentally expose an internal service:

```bash
# 1. Edit docker-compose production.yml
# Change from:
#   ports:
#     - "50051:50051"
# To:
#   expose:
#     - "50051"

# 2. Recreate the service
docker-compose -f .sutra/compose/production.yml up -d --force-recreate <service-name>

# 3. Verify port is no longer exposed
docker ps | grep <service-name>
```

---

## Performance Tuning

### Connection Limits

Adjust worker connections based on traffic:

```nginx
events {
    worker_connections 2048;  # Default, increase for high traffic
    use epoll;
    multi_accept on;
}
```

### Keepalive Settings

```nginx
keepalive_timeout 65;
keepalive_requests 100;
```

### Buffer Sizes

For high-throughput APIs:

```nginx
client_body_buffer_size 128k;
client_max_body_size 100m;  # Adjust based on upload size needs
proxy_buffer_size 4k;
proxy_buffers 8 4k;
```

---

## Compliance & Auditing

### Security Audit Checklist

- [ ] All internal services use `expose:` instead of `ports:`
- [ ] Only nginx-proxy exposes ports 80, 443, 8080
- [ ] SSL certificates are valid and up-to-date
- [ ] HSTS header enabled in production
- [ ] Rate limiting configured appropriately
- [ ] Internal metrics endpoints restricted to internal networks
- [ ] Authentication enabled (`SUTRA_SECURE_MODE=true`)
- [ ] Security headers present in all responses
- [ ] Nginx logs are being collected and monitored
- [ ] Firewall rules only allow 80, 443 (and 8080 for dev)

### Generating Security Report

```bash
# Network exposure audit
./scripts/audit-network-exposure.sh

# Naming convention validation
./scripts/validate-naming.sh

# SSL/TLS configuration test
docker exec sutra-works-nginx-proxy openssl s_client -connect localhost:443 -tls1_3

# Security headers test
curl -I https://localhost/api/health | grep -E "(X-|Strict-Transport)"

# Rate limit test
for i in {1..100}; do curl http://localhost:8080/api/health; done
```

---

## Migration from Exposed Services

If you're migrating from the old configuration with exposed services:

```bash
# 1. Backup current configuration
cp .sutra/compose/production.yml .sutra/compose/production.yml.backup

# 2. Update to new configuration with nginx proxy
git pull origin main  # Or apply the changes manually

# 3. Stop all services
docker-compose -f .sutra/compose/production.yml down

# 4. Remove old containers (optional, if you want clean state)
docker-compose -f .sutra/compose/production.yml rm -f

# 5. Build nginx proxy
docker-compose -f .sutra/compose/production.yml build nginx-proxy

# 6. Start with new configuration
docker-compose -f .sutra/compose/production.yml up -d

# 7. Verify services are NOT exposed
docker ps --format "table {{.Names}}\t{{.Ports}}"

# 8. Update client applications to use nginx proxy
# Old: http://localhost:8000/health
# New: http://localhost:8080/api/health (or https://yourdomain.com/api/health)
```

---

## Support & References

- **Main Documentation**: [docs/deployment/PRODUCTION_DEPLOYMENT_GUIDE.md](./PRODUCTION_DEPLOYMENT_GUIDE.md)
- **Security Setup**: [docs/security/PRODUCTION_SECURITY_SETUP.md](../security/PRODUCTION_SECURITY_SETUP.md)
- **Architecture**: [docs/architecture/SYSTEM_ARCHITECTURE.md](../architecture/SYSTEM_ARCHITECTURE.md)

For issues or questions:
- Security issues: security@sutra-ai.dev
- GitHub Issues: https://github.com/nranjan2code/sutra-memory/issues

**Last Updated:** 2025-01-06
