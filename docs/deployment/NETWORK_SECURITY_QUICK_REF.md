# Network Security Quick Reference

## TL;DR

All external traffic now routes through nginx reverse proxy. Internal services (storage, ML, grid) are NOT exposed to the host network.

## Single Entry Point Architecture

```
Internet → Nginx Proxy (80/443/8080) → Internal Services
```

## Exposed Ports (Host)

| Port | Protocol | Service | Purpose |
|------|----------|---------|---------|
| 80 | HTTP | nginx-proxy | Redirects to HTTPS (production) |
| 443 | HTTPS | nginx-proxy | Main entry point (production) |
| 8080 | HTTP | nginx-proxy | Development (no SSL) |

**ALL other services are internal-only**

## Service Access via Nginx

### API Endpoints

| Old (Direct) | New (via Nginx) |
|--------------|-----------------|
| http://localhost:8000/... | http://localhost:8080/api/... |
| http://localhost:8001/sutra/... | http://localhost:8080/sutra/... |
| http://localhost:8005/... | http://localhost:8080/ingest/... |

### Web UIs

| Service | URL (via Nginx) |
|---------|-----------------|
| Main Client | http://localhost:8080/ |
| Control Center | http://localhost:8080/control/ |
| Explorer | http://localhost:8080/explorer/ |
| Explorer API | http://localhost:8080/explorer-api/ |

## Quick Deployment

```bash
# Development
docker-compose -f .sutra/compose/production.yml up -d

# Access services
curl http://localhost:8080/health
curl http://localhost:8080/api/health

# Production (with SSL)
export SUTRA_SSL_CERT_PATH=/path/to/cert.pem
export SUTRA_SSL_KEY_PATH=/path/to/key.pem
docker-compose -f .sutra/compose/production.yml up -d

# Access services
curl https://yourdomain.com/health
curl https://yourdomain.com/api/health
```

## Security Checks

```bash
# Verify no services expose ports (except nginx)
docker ps --format "table {{.Names}}\t{{.Ports}}" | grep -v nginx-proxy

# Should show NO host port mappings for other services

# Run comprehensive audit
./scripts/audit-network-exposure.sh
```

## Migration Checklist

- [ ] Update `.sutra/compose/production.yml` (all internal services use `expose:` not `ports:`)
- [ ] Add nginx-proxy service to compose file
- [ ] Update client applications to use nginx proxy URLs
- [ ] Configure SSL certificates for production
- [ ] Test all endpoints via nginx proxy
- [ ] Remove old port mappings from firewall rules
- [ ] Update documentation/scripts with new URLs

## Troubleshooting

### Service not accessible

```bash
# Check nginx is running
docker ps | grep nginx-proxy

# Check nginx config
docker exec sutra-nginx-proxy nginx -t

# Check backend service
docker exec sutra-nginx-proxy curl http://sutra-api:8000/health
```

### Port conflicts

```bash
# Check what's using port 80/443/8080
sudo lsof -i :80
sudo lsof -i :443
sudo lsof -i :8080

# Stop conflicting services or change nginx ports
```

## Rate Limits

| Endpoint | Limit | Burst |
|----------|-------|-------|
| /api/auth | 10/min | 5 |
| /api/* | 60/min | 10 |
| /ingest/* | 60/min | 5 |
| /* (general) | 120/min | 20 |

## Internal Services (NOT Exposed)

These are accessible only within Docker network:

- storage-server:50051
- grid-event-storage:50051
- user-storage-server:50051
- grid-master:7001, 7002
- grid-agent-1, grid-agent-2:8001
- ml-base-service:8887
- embedding-single/ha:8888
- nlg-single/ha:8003, 8889

## Full Documentation

- **Complete Guide**: [NETWORK_SECURITY.md](./NETWORK_SECURITY.md)
- **Production Deployment**: [PRODUCTION_DEPLOYMENT_GUIDE.md](./PRODUCTION_DEPLOYMENT_GUIDE.md)
- **Security Setup**: [../security/PRODUCTION_SECURITY_SETUP.md](../security/PRODUCTION_SECURITY_SETUP.md)
