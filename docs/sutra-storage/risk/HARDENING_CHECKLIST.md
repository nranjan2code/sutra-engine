# Security Hardening Checklist - Sutra Storage

**Document Purpose**: Step-by-step hardening guide for production deployment  
**Completion Time**: 2-3 hours  
**Validation**: Each item includes verification step

---

## Pre-Deployment Security Hardening

### 1. Authentication & Authorization

#### ✅ 1.1 Enable Secure Mode
**Action Required**: Set secure mode environment variable
```bash
export SUTRA_SECURE_MODE=true
```

**Verification**:
```bash
# Check environment variable is set
echo "Secure mode: ${SUTRA_SECURE_MODE:-DISABLED}"

# Verify server starts in secure mode  
grep "SECURE" /var/log/sutra/storage.log
```

**Status**: [ ] Complete

---

#### ✅ 1.2 Generate Strong Authentication Secret
**Action Required**: Create cryptographically secure 64+ character secret
```bash
# Generate secure secret
openssl rand -hex 32 > /etc/sutra/auth-secret

# Set restrictive permissions
chmod 600 /etc/sutra/auth-secret
chown sutra:sutra /etc/sutra/auth-secret

# Configure environment
export SUTRA_AUTH_SECRET=$(cat /etc/sutra/auth-secret)
```

**Verification**:
```bash
# Verify secret length (should be ≥64 characters)
SECRET_LENGTH=$(cat /etc/sutra/auth-secret | wc -c)
if [ $SECRET_LENGTH -ge 64 ]; then
    echo "✅ Secret length: $SECRET_LENGTH characters"
else  
    echo "❌ Secret too short: $SECRET_LENGTH characters"
fi

# Verify permissions
ls -la /etc/sutra/auth-secret | grep -E "^-rw-------.*sutra sutra"
```

**Status**: [ ] Complete

---

#### ✅ 1.3 Configure Role-Based Access Control
**Action Required**: Define user roles and permissions
```bash
# Create user configuration file
cat > /etc/sutra/users.json << 'EOF'
{
  "users": {
    "admin": {
      "roles": ["Admin"],
      "description": "System administrator"
    },
    "api-service": {
      "roles": ["Writer"],  
      "description": "API service account"
    },
    "read-only": {
      "roles": ["Reader"],
      "description": "Read-only access"
    }
  }
}
EOF

chmod 640 /etc/sutra/users.json
chown sutra:sutra /etc/sutra/users.json
```

**Verification**:
```bash
# Verify user configuration is valid JSON
cat /etc/sutra/users.json | jq '.' > /dev/null && echo "✅ Valid user config" || echo "❌ Invalid JSON"

# Test role-based authentication (requires server running)
python3 -c "
import json
users = json.load(open('/etc/sutra/users.json'))
for user, config in users['users'].items():
    print(f'✅ User: {user}, Roles: {config[\"roles\"]}')
"
```

**Status**: [ ] Complete

---

### 2. Network Security

#### ✅ 2.1 Configure TLS Encryption
**Action Required**: Generate or deploy TLS certificates
```bash
# For development - generate self-signed certificate
mkdir -p /etc/sutra/certs
cd /etc/sutra/certs

# Generate private key
openssl genrsa -out server-key.pem 4096

# Generate certificate signing request
openssl req -new -key server-key.pem -out server-csr.pem \
    -subj "/C=US/ST=CA/L=San Francisco/O=Sutra AI/CN=sutra-storage"

# Generate self-signed certificate (365 days)
openssl x509 -req -days 365 -in server-csr.pem \
    -signkey server-key.pem -out server-cert.pem

# Set permissions
chmod 600 server-key.pem
chmod 644 server-cert.pem  
chown -R sutra:sutra /etc/sutra/certs
```

**Production Alternative** (Let's Encrypt):
```bash
# Install certbot
sudo apt-get install certbot

# Generate certificate for domain
certbot certonly --standalone -d sutra-storage.yourdomain.com

# Link certificates  
ln -sf /etc/letsencrypt/live/sutra-storage.yourdomain.com/fullchain.pem /etc/sutra/certs/server-cert.pem
ln -sf /etc/letsencrypt/live/sutra-storage.yourdomain.com/privkey.pem /etc/sutra/certs/server-key.pem
```

**Configuration**:
```bash
export SUTRA_TLS_ENABLED=true
export SUTRA_TLS_CERT=/etc/sutra/certs/server-cert.pem
export SUTRA_TLS_KEY=/etc/sutra/certs/server-key.pem
```

**Verification**:
```bash
# Verify certificate validity
openssl x509 -in /etc/sutra/certs/server-cert.pem -noout -dates
openssl x509 -in /etc/sutra/certs/server-cert.pem -noout -subject
openssl x509 -in /etc/sutra/certs/server-cert.pem -noout -issuer

# Test TLS handshake (requires server running)
echo | openssl s_client -connect localhost:50051 -servername sutra-storage 2>/dev/null | grep -E "^verify|subject="
```

**Status**: [ ] Complete

---

#### ✅ 2.2 Configure Firewall Rules
**Action Required**: Restrict network access to authorized sources
```bash
# Enable UFW firewall
ufw --force enable

# Default deny incoming
ufw default deny incoming
ufw default allow outgoing

# Allow SSH (if needed)
ufw allow ssh

# Allow Sutra storage only from internal networks
ufw allow from 10.0.0.0/8 to any port 50051 comment 'Sutra storage - internal'
ufw allow from 172.16.0.0/12 to any port 50051 comment 'Sutra storage - docker'
ufw allow from 192.168.0.0/16 to any port 50051 comment 'Sutra storage - private'

# Allow API access (if separate server)
ufw allow from 10.0.0.0/8 to any port 8000 comment 'Sutra API - internal'

# Deny all other access to Sutra ports
ufw deny 50051 comment 'Block external Sutra storage'
ufw deny 8000 comment 'Block external Sutra API'
```

**Verification**:
```bash
# Check firewall status
ufw status numbered

# Test external access is blocked (should fail)
timeout 5 nc -z PUBLIC_IP 50051 && echo "❌ External access allowed" || echo "✅ External access blocked"

# Test internal access works (should succeed)  
timeout 5 nc -z 127.0.0.1 50051 && echo "✅ Internal access works" || echo "❌ Internal access blocked"
```

**Status**: [ ] Complete

---

#### ✅ 2.3 Configure Rate Limiting
**Action Required**: Set up connection and request rate limits
```bash
# Configure iptables rate limiting (additional protection)
iptables -A INPUT -p tcp --dport 50051 -m connlimit --connlimit-above 20 --connlimit-mask 24 -j DROP
iptables -A INPUT -p tcp --dport 50051 -m hashlimit --hashlimit-mode srcip --hashlimit 100/min --hashlimit-name storage-rate -j ACCEPT
iptables -A INPUT -p tcp --dport 8000 -m connlimit --connlimit-above 50 --connlimit-mask 24 -j DROP
iptables -A INPUT -p tcp --dport 8000 -m hashlimit --hashlimit-mode srcip --hashlimit 200/min --hashlimit-name api-rate -j ACCEPT

# Save iptables rules
iptables-save > /etc/iptables/rules.v4
```

**API Rate Limiting Configuration**:
```bash
# Configure application-level rate limiting
cat > /etc/sutra/rate-limits.json << 'EOF'
{
  "endpoints": {
    "/learn": {"limit": 60, "window": 60},
    "/learn/batch": {"limit": 30, "window": 60},
    "/query": {"limit": 200, "window": 60},
    "/search": {"limit": 300, "window": 60}
  },
  "global_limit": 100,
  "trusted_proxies": [
    "10.0.0.0/8",
    "172.16.0.0/12", 
    "192.168.0.0/16"
  ]
}
EOF
```

**Verification**:
```bash
# Test rate limiting (should eventually get 429 responses)
for i in {1..150}; do
    curl -s -o /dev/null -w "%{http_code} " http://localhost:8000/health
    if [ $((i % 10)) -eq 0 ]; then echo ""; fi
done

# Check iptables rules
iptables -L INPUT -n --line-numbers | grep -E "(50051|8000)"
```

**Status**: [ ] Complete

---

### 3. Application Security

#### ✅ 3.1 Input Validation Hardening
**Action Required**: Enable strict input validation
```bash
# Configure strict validation settings
cat > /etc/sutra/validation.toml << 'EOF'
[validation]
max_content_size = 1048576        # 1MB maximum content size
max_query_length = 1000           # Maximum query string length
max_metadata_size = 4096          # Maximum metadata size
allow_html = false                # Disable HTML in content
require_utf8 = true               # Require valid UTF-8
validate_embeddings = true        # Validate embedding dimensions

[sanitization]
strip_control_chars = true        # Remove control characters
normalize_unicode = true          # Normalize Unicode text
max_line_length = 1000           # Maximum line length
EOF

chmod 640 /etc/sutra/validation.toml
chown sutra:sutra /etc/sutra/validation.toml
```

**Verification**:
```bash
# Test input validation with oversized content (should be rejected)
python3 -c "
import requests
import json

# Test oversized content
oversized = 'A' * 2000000  # 2MB content
try:
    resp = requests.post('http://localhost:8000/learn', 
                        json={'content': oversized})
    print(f'Oversized content: {resp.status_code}')
except Exception as e:
    print(f'Request failed (expected): {e}')

# Test malicious content
malicious = '<script>alert(1)</script>'
try:
    resp = requests.post('http://localhost:8000/learn',
                        json={'content': malicious})
    print(f'Malicious content: {resp.status_code}')
except Exception as e:
    print(f'Request failed: {e}')
"
```

**Status**: [ ] Complete

---

#### ✅ 3.2 Secure Deserialization
**Action Required**: Configure MessagePack security settings
```bash
# Configure secure deserialization limits
cat >> /etc/sutra/storage.toml << 'EOF'
[protocol]
max_message_size = 10485760       # 10MB maximum message size
max_array_length = 10000          # Maximum array elements
max_map_length = 1000             # Maximum map entries  
max_str_length = 1048576          # 1MB maximum string length
max_bin_length = 10485760         # 10MB maximum binary length
max_ext_length = 1048576          # 1MB maximum extension length

[security]
strict_parsing = true             # Enable strict MessagePack parsing
validate_types = true             # Validate message types
timeout_seconds = 30              # Request timeout
EOF
```

**Verification**:
```bash
# Test message size limits (should be rejected)
python3 -c "
import socket
import msgpack
import struct

# Test oversized message
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.settimeout(5)

try:
    sock.connect(('localhost', 50051))
    
    # Create oversized message
    large_data = {'content': 'A' * 20000000}  # 20MB
    payload = msgpack.packb(large_data)
    
    # Send message length and payload
    sock.send(struct.pack('>I', len(payload)))
    sock.send(payload[:1024])  # Send partial to test limits
    
    response = sock.recv(1024)
    print(f'Server response length: {len(response)}')
    
except Exception as e:
    print(f'Connection failed (expected for oversized): {e}')
finally:
    sock.close()
"
```

**Status**: [ ] Complete

---

#### ✅ 3.3 Secure File Operations  
**Action Required**: Harden file system permissions and storage
```bash
# Create secure storage directory structure
mkdir -p /var/lib/sutra/{data,wal,index,tmp}
chmod 750 /var/lib/sutra
chmod 700 /var/lib/sutra/{data,wal,index}
chmod 1777 /var/lib/sutra/tmp  # Sticky bit for tmp
chown -R sutra:sutra /var/lib/sutra

# Set secure umask for sutra processes  
echo "umask 027" >> /etc/sutra/environment

# Configure storage security settings
cat > /etc/sutra/storage-security.toml << 'EOF'
[storage]
data_directory = "/var/lib/sutra/data"
wal_directory = "/var/lib/sutra/wal"
index_directory = "/var/lib/sutra/index"
temp_directory = "/var/lib/sutra/tmp"

[security]
file_permissions = "0640"         # Owner read/write, group read
directory_permissions = "0750"    # Owner full, group read/execute
disable_symlinks = true           # Prevent symlink attacks
validate_paths = true             # Validate all file paths
enable_fsync = true               # Force disk synchronization

[backup]
backup_directory = "/var/backups/sutra"
backup_permissions = "0600"       # Backup files read-only to owner
backup_retention_days = 30        # Retain backups for 30 days
EOF
```

**Verification**:
```bash
# Check directory permissions
ls -la /var/lib/sutra/
find /var/lib/sutra -type d -exec ls -ld {} \;

# Verify file creation permissions
sudo -u sutra touch /var/lib/sutra/data/test-file
ls -la /var/lib/sutra/data/test-file
rm /var/lib/sutra/data/test-file

# Test symlink prevention (should fail)
ln -s /etc/passwd /var/lib/sutra/data/symlink-test 2>/dev/null && echo "❌ Symlinks allowed" || echo "✅ Symlinks blocked"
```

**Status**: [ ] Complete

---

### 4. Monitoring & Logging

#### ✅ 4.1 Security Event Logging
**Action Required**: Configure comprehensive security logging
```bash
# Configure security logging
mkdir -p /var/log/sutra
chown sutra:sutra /var/log/sutra
chmod 750 /var/log/sutra

# Create log rotation configuration
cat > /etc/logrotate.d/sutra << 'EOF'
/var/log/sutra/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0640 sutra sutra
    postrotate
        systemctl reload sutra-storage 2>/dev/null || true
    endscript
}
EOF

# Configure structured logging
cat > /etc/sutra/logging.toml << 'EOF'
[logging]
level = "info"
format = "json"
output = "/var/log/sutra/storage.log"

[security_logging]
enabled = true
log_file = "/var/log/sutra/security.log"
events = [
    "authentication_attempt",
    "authentication_failure", 
    "authorization_failure",
    "rate_limit_exceeded",
    "invalid_request",
    "connection_rejected",
    "tls_handshake_failure"
]

[audit_logging]
enabled = true
log_file = "/var/log/sutra/audit.log"
events = [
    "learn_concept",
    "query_graph", 
    "flush_storage",
    "admin_action"
]
EOF
```

**Verification**:
```bash
# Test log file creation and permissions
sudo -u sutra touch /var/log/sutra/{storage,security,audit}.log
ls -la /var/log/sutra/

# Test log rotation configuration  
logrotate -d /etc/logrotate.d/sutra

# Check security events are logged (requires server running)
tail -f /var/log/sutra/security.log | jq '.' || echo "Waiting for log entries..."
```

**Status**: [ ] Complete

---

#### ✅ 4.2 Monitoring Integration
**Action Required**: Set up security monitoring and alerting
```bash
# Install monitoring tools
apt-get update && apt-get install -y fail2ban auditd

# Configure fail2ban for Sutra
cat > /etc/fail2ban/filter.d/sutra.conf << 'EOF'
[Definition]
failregex = Authentication failed.*from <HOST>
            Rate limit exceeded.*from <HOST>
            Invalid request.*from <HOST>
            TLS handshake failure.*from <HOST>

ignoreregex = 
EOF

cat > /etc/fail2ban/jail.d/sutra.conf << 'EOF'
[sutra]
enabled = true
port = 50051,8000
filter = sutra
logpath = /var/log/sutra/security.log
maxretry = 5
findtime = 300
bantime = 3600
action = iptables-multiport[name=sutra, port="50051,8000", protocol=tcp]
EOF

# Configure audit rules for Sutra files
cat >> /etc/audit/rules.d/sutra.rules << 'EOF'
# Monitor Sutra configuration files
-w /etc/sutra/ -p wa -k sutra-config

# Monitor Sutra data directory
-w /var/lib/sutra/ -p wa -k sutra-data

# Monitor Sutra binaries
-w /usr/local/bin/storage-server -p x -k sutra-exec
-w /usr/local/bin/sutra-api -p x -k sutra-exec

# Monitor authentication events
-w /var/log/sutra/security.log -p wa -k sutra-security
EOF

# Restart services
systemctl restart fail2ban
systemctl restart auditd
```

**Verification**:
```bash
# Check fail2ban status
fail2ban-client status sutra

# Test audit rules
auditctl -l | grep sutra

# Generate test security events and verify monitoring
python3 -c "
import socket
import time

# Test failed authentication (should trigger fail2ban)
for i in range(3):
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(2)
        sock.connect(('localhost', 50051))
        sock.send(b'invalid-auth-token')
        sock.close()
        time.sleep(1)
    except:
        pass
        
print('Generated test authentication failures')
"

# Check if events were detected
fail2ban-client status sutra
```

**Status**: [ ] Complete

---

#### ✅ 4.3 Health Check Automation
**Action Required**: Deploy automated security health monitoring
```bash
# Create security health check script
cat > /usr/local/bin/sutra-security-check << 'EOF'
#!/bin/bash
set -euo pipefail

LOG_FILE="/var/log/sutra/health-check.log"
ALERT_EMAIL="security@yourdomain.com"

log() {
    echo "$(date -Iseconds) $1" | tee -a "$LOG_FILE"
}

alert() {
    log "ALERT: $1"
    echo "$1" | mail -s "Sutra Security Alert" "$ALERT_EMAIL" 2>/dev/null || true
}

check_security_mode() {
    if [ "${SUTRA_SECURE_MODE:-false}" != "true" ]; then
        alert "CRITICAL: Secure mode is disabled"
        return 1
    fi
    log "✅ Secure mode: enabled"
}

check_auth_secret() {
    if [ -z "${SUTRA_AUTH_SECRET:-}" ]; then
        alert "CRITICAL: Authentication secret not configured"  
        return 1
    fi
    
    if [ ${#SUTRA_AUTH_SECRET} -lt 32 ]; then
        alert "CRITICAL: Authentication secret too short"
        return 1
    fi
    log "✅ Authentication secret: configured"
}

check_tls() {
    if [ "${SUTRA_TLS_ENABLED:-false}" != "true" ]; then
        log "WARNING: TLS encryption disabled"
    else
        # Check certificate expiry
        cert_file="${SUTRA_TLS_CERT:-/etc/sutra/certs/server-cert.pem}"
        if [ -f "$cert_file" ]; then
            expiry_date=$(openssl x509 -in "$cert_file" -noout -enddate | cut -d= -f2)
            expiry_epoch=$(date -d "$expiry_date" +%s)
            current_epoch=$(date +%s)
            days_until_expiry=$(( (expiry_epoch - current_epoch) / 86400 ))
            
            if [ $days_until_expiry -lt 30 ]; then
                alert "WARNING: TLS certificate expires in $days_until_expiry days"
            else
                log "✅ TLS certificate: valid for $days_until_expiry days"
            fi
        else
            alert "ERROR: TLS certificate file not found: $cert_file"
        fi
    fi
}

check_firewall() {
    if ! ufw status | grep -q "Status: active"; then
        alert "WARNING: UFW firewall not active"
    else
        log "✅ Firewall: active"
    fi
}

check_fail2ban() {
    if ! systemctl is-active --quiet fail2ban; then
        alert "WARNING: fail2ban not running"
    else
        banned_ips=$(fail2ban-client status sutra 2>/dev/null | grep "Banned IP list:" | wc -w)
        log "✅ fail2ban: active ($banned_ips banned IPs)"
    fi
}

check_log_files() {
    for log in security.log audit.log storage.log; do
        log_path="/var/log/sutra/$log"
        if [ ! -f "$log_path" ]; then
            alert "WARNING: Log file missing: $log_path"
        else
            # Check for recent activity (last 24 hours)
            if find "$log_path" -mtime -1 | grep -q .; then
                log "✅ Log file active: $log"
            else
                log "WARNING: No recent activity in $log"
            fi
        fi
    done
}

check_storage_connectivity() {
    if timeout 5 nc -z localhost 50051 2>/dev/null; then
        log "✅ Storage server: responding"
    else
        alert "CRITICAL: Storage server not responding"
        return 1
    fi
}

# Run all checks
log "Starting security health check"

failed_checks=0
check_security_mode || ((failed_checks++))
check_auth_secret || ((failed_checks++))
check_tls || true  # Non-critical
check_firewall || true  # Non-critical
check_fail2ban || true  # Non-critical  
check_log_files || true  # Non-critical
check_storage_connectivity || ((failed_checks++))

if [ $failed_checks -eq 0 ]; then
    log "✅ Security health check: PASSED"
    exit 0
else
    alert "❌ Security health check: FAILED ($failed_checks critical issues)"
    exit 1
fi
EOF

chmod +x /usr/local/bin/sutra-security-check

# Create systemd service for automated checks
cat > /etc/systemd/system/sutra-security-check.service << 'EOF'
[Unit]
Description=Sutra Security Health Check
After=sutra-storage.service

[Service]
Type=oneshot
User=sutra
ExecStart=/usr/local/bin/sutra-security-check
EnvironmentFile=/etc/sutra/environment
EOF

# Create systemd timer for regular execution
cat > /etc/systemd/system/sutra-security-check.timer << 'EOF'  
[Unit]
Description=Run Sutra Security Health Check
Requires=sutra-security-check.service

[Timer]
OnCalendar=*:0/30  # Every 30 minutes
Persistent=true

[Install]
WantedBy=timers.target
EOF

# Enable and start timer
systemctl daemon-reload
systemctl enable sutra-security-check.timer
systemctl start sutra-security-check.timer
```

**Verification**:
```bash
# Test health check script manually
/usr/local/bin/sutra-security-check

# Check timer status
systemctl status sutra-security-check.timer
systemctl list-timers sutra-security-check.timer

# View health check logs
tail -f /var/log/sutra/health-check.log
```

**Status**: [ ] Complete

---

### 5. Backup & Recovery

#### ✅ 5.1 Security Configuration Backup
**Action Required**: Implement automated backup of security configurations
```bash
# Create backup script
cat > /usr/local/bin/sutra-security-backup << 'EOF'
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/var/backups/sutra/security"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/security-config-$DATE.tar.gz.gpg"

# Create backup directory
mkdir -p "$BACKUP_DIR"
chmod 700 "$BACKUP_DIR"

# Create temporary staging directory  
STAGING_DIR=$(mktemp -d)
trap "rm -rf $STAGING_DIR" EXIT

# Copy security configurations
cp -r /etc/sutra/ "$STAGING_DIR/"
cp -r /etc/ssl/private/sutra* "$STAGING_DIR/sutra/" 2>/dev/null || true

# Remove sensitive files from staging (will be backed up separately)
rm -f "$STAGING_DIR/sutra/auth-secret"

# Create encrypted backup
tar -czf - -C "$STAGING_DIR" . | gpg --cipher-algo AES256 --compress-algo 1 --symmetric --output "$BACKUP_FILE"

# Set secure permissions
chmod 600 "$BACKUP_FILE"

# Create checksum
sha256sum "$BACKUP_FILE" > "$BACKUP_FILE.sha256"

# Clean up old backups (keep 30 days)  
find "$BACKUP_DIR" -name "security-config-*.tar.gz.gpg" -mtime +30 -delete
find "$BACKUP_DIR" -name "*.sha256" -mtime +30 -delete

echo "Security configuration backed up to: $BACKUP_FILE"
EOF

chmod +x /usr/local/bin/sutra-security-backup

# Create backup service
cat > /etc/systemd/system/sutra-security-backup.service << 'EOF'
[Unit]
Description=Sutra Security Configuration Backup

[Service]
Type=oneshot
User=root
ExecStart=/usr/local/bin/sutra-security-backup
EOF

# Create backup timer (daily at 2 AM)
cat > /etc/systemd/system/sutra-security-backup.timer << 'EOF'
[Unit]
Description=Daily Sutra Security Backup
Requires=sutra-security-backup.service

[Timer]
OnCalendar=daily
Persistent=true
RandomizedDelaySec=600

[Install]  
WantedBy=timers.target
EOF

# Enable backup timer
systemctl daemon-reload
systemctl enable sutra-security-backup.timer
systemctl start sutra-security-backup.timer
```

**Verification**:
```bash
# Test backup script
/usr/local/bin/sutra-security-backup

# Verify backup was created
ls -la /var/backups/sutra/security/

# Test backup restoration (dry run)
LATEST_BACKUP=$(ls -t /var/backups/sutra/security/security-config-*.tar.gz.gpg | head -n1)
echo "Latest backup: $LATEST_BACKUP"
gpg --decrypt "$LATEST_BACKUP" | tar -tzf - | head -10
```

**Status**: [ ] Complete

---

#### ✅ 5.2 Secret Recovery Procedures
**Action Required**: Document and test secret recovery procedures
```bash
# Create secret recovery script
cat > /usr/local/bin/sutra-secret-recovery << 'EOF'
#!/bin/bash
set -euo pipefail

RECOVERY_DIR="/var/recovery/sutra"
DATE=$(date +%Y%m%d_%H%M%S)

usage() {
    echo "Usage: $0 {backup|restore|rotate}"
    echo "  backup  - Backup current secrets"
    echo "  restore - Restore secrets from backup" 
    echo "  rotate  - Generate new secrets and backup old ones"
    exit 1
}

backup_secrets() {
    echo "Backing up secrets..."
    mkdir -p "$RECOVERY_DIR"
    
    # Backup auth secret
    if [ -f /etc/sutra/auth-secret ]; then
        cp /etc/sutra/auth-secret "$RECOVERY_DIR/auth-secret-$DATE"
        chmod 600 "$RECOVERY_DIR/auth-secret-$DATE"
    fi
    
    # Backup TLS certificates
    if [ -d /etc/sutra/certs ]; then
        tar -czf "$RECOVERY_DIR/certs-$DATE.tar.gz" -C /etc/sutra certs/
        chmod 600 "$RECOVERY_DIR/certs-$DATE.tar.gz"
    fi
    
    echo "Secrets backed up with timestamp: $DATE"
}

restore_secrets() {
    echo "Available secret backups:"
    ls -la "$RECOVERY_DIR/" | grep -E "(auth-secret|certs-)" || {
        echo "No backup files found in $RECOVERY_DIR"
        exit 1
    }
    
    echo -n "Enter timestamp to restore (YYYYMMDD_HHMMSS): "
    read -r timestamp
    
    # Restore auth secret
    if [ -f "$RECOVERY_DIR/auth-secret-$timestamp" ]; then
        cp "$RECOVERY_DIR/auth-secret-$timestamp" /etc/sutra/auth-secret
        chmod 600 /etc/sutra/auth-secret
        chown sutra:sutra /etc/sutra/auth-secret
        echo "Auth secret restored"
    fi
    
    # Restore certificates
    if [ -f "$RECOVERY_DIR/certs-$timestamp.tar.gz" ]; then
        tar -xzf "$RECOVERY_DIR/certs-$timestamp.tar.gz" -C /etc/sutra/
        chown -R sutra:sutra /etc/sutra/certs/
        echo "Certificates restored"
    fi
    
    echo "Secret restoration complete. Restart services to apply."
}

rotate_secrets() {
    echo "Rotating secrets..."
    
    # Backup current secrets
    backup_secrets
    
    # Generate new auth secret
    openssl rand -hex 32 > /etc/sutra/auth-secret.new
    chmod 600 /etc/sutra/auth-secret.new
    chown sutra:sutra /etc/sutra/auth-secret.new
    
    # Move new secret into place
    mv /etc/sutra/auth-secret.new /etc/sutra/auth-secret
    
    echo "New auth secret generated. Update SUTRA_AUTH_SECRET environment variable."
    echo "New secret: $(cat /etc/sutra/auth-secret)"
    
    # Note: Certificate rotation would need to be done separately
    echo "Note: TLS certificate rotation must be done manually"
}

case "${1:-}" in
    backup)
        backup_secrets
        ;;
    restore)
        restore_secrets
        ;;
    rotate)
        rotate_secrets
        ;;
    *)
        usage
        ;;
esac
EOF

chmod +x /usr/local/bin/sutra-secret-recovery
```

**Verification**:
```bash
# Test secret backup
/usr/local/bin/sutra-secret-recovery backup

# Verify backup files
ls -la /var/recovery/sutra/

# Test secret rotation (will generate new secret)
echo "Current secret (first 16 chars): $(cat /etc/sutra/auth-secret | head -c 16)..."
/usr/local/bin/sutra-secret-recovery rotate
echo "New secret (first 16 chars): $(cat /etc/sutra/auth-secret | head -c 16)..."
```

**Status**: [ ] Complete

---

## Post-Deployment Verification

### ✅ 6.1 Comprehensive Security Test
**Action Required**: Run complete security validation
```bash
# Create comprehensive security test
cat > /usr/local/bin/sutra-security-test << 'EOF'
#!/bin/bash
set -euo pipefail

FAILED_TESTS=0
TOTAL_TESTS=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_result="$3"  # "success" or "failure"
    
    ((TOTAL_TESTS++))
    echo -n "Testing $test_name... "
    
    if eval "$test_command" >/dev/null 2>&1; then
        if [ "$expected_result" = "success" ]; then
            echo "✅ PASS"
        else
            echo "❌ FAIL (expected failure but succeeded)"
            ((FAILED_TESTS++))
        fi
    else
        if [ "$expected_result" = "failure" ]; then
            echo "✅ PASS (correctly failed)"  
        else
            echo "❌ FAIL (expected success but failed)"
            ((FAILED_TESTS++))
        fi
    fi
}

echo "🔒 Sutra Security Test Suite"
echo "============================"

# Test 1: Verify secure mode is enabled
run_test "Secure mode enabled" '[ "$SUTRA_SECURE_MODE" = "true" ]' "success"

# Test 2: Verify auth secret is configured  
run_test "Auth secret configured" '[ -n "$SUTRA_AUTH_SECRET" ] && [ ${#SUTRA_AUTH_SECRET} -ge 32 ]' "success"

# Test 3: Verify TLS certificates exist
run_test "TLS certificates present" '[ -f "/etc/sutra/certs/server-cert.pem" ] && [ -f "/etc/sutra/certs/server-key.pem" ]' "success"

# Test 4: Verify storage server is running
run_test "Storage server connectivity" 'timeout 5 nc -z localhost 50051' "success"

# Test 5: Verify unauthenticated access is blocked
run_test "Unauthenticated access blocked" 'timeout 2 echo "test" | nc localhost 50051' "failure"

# Test 6: Verify firewall is active
run_test "Firewall active" 'ufw status | grep -q "Status: active"' "success"

# Test 7: Verify fail2ban is running
run_test "fail2ban service active" 'systemctl is-active --quiet fail2ban' "success"

# Test 8: Verify log files exist and are writable
run_test "Security logs writable" 'sudo -u sutra touch /var/log/sutra/test.log && rm /var/log/sutra/test.log' "success"

# Test 9: Verify backup directories exist  
run_test "Backup directories configured" '[ -d "/var/backups/sutra" ] && [ -d "/var/recovery/sutra" ]' "success"

# Test 10: Verify file permissions are secure
run_test "Auth secret permissions" '[ "$(stat -c %a /etc/sutra/auth-secret)" = "600" ]' "success"

echo ""
echo "Test Results:"
echo "============="
echo "Total tests: $TOTAL_TESTS"
echo "Passed: $((TOTAL_TESTS - FAILED_TESTS))"
echo "Failed: $FAILED_TESTS"

if [ $FAILED_TESTS -eq 0 ]; then
    echo ""
    echo "🎉 All security tests passed!"
    echo "Sutra storage is properly hardened for production deployment."
    exit 0
else
    echo ""
    echo "❌ Security test failures detected!"
    echo "Please review and fix the failed tests before deployment."
    exit 1
fi
EOF

chmod +x /usr/local/bin/sutra-security-test
```

**Verification**:
```bash
# Run comprehensive security test
/usr/local/bin/sutra-security-test

# Expected output should show all tests passing
```

**Status**: [ ] Complete

---

### ✅ 6.2 Penetration Testing
**Action Required**: Run basic penetration tests to validate security
```bash
# Create penetration test script
cat > /usr/local/bin/sutra-pentest << 'EOF'
#!/bin/bash
set -euo pipefail

echo "🏴‍☠️ Sutra Security Penetration Test"
echo "====================================="

# Test 1: Authentication bypass attempt
echo "1. Testing authentication bypass..."
python3 -c "
import socket
import time

# Test 1a: Direct connection without auth
try:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(3)
    sock.connect(('localhost', 50051))
    sock.send(b'test message')
    response = sock.recv(1024)
    print('❌ VULNERABILITY: Unauthenticated access allowed')
except:
    print('✅ Authentication required (expected)')

# Test 1b: Invalid token format
try:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)  
    sock.settimeout(3)
    sock.connect(('localhost', 50051))
    sock.send(b'Bearer invalid-token-format')
    response = sock.recv(1024)
    print('❌ VULNERABILITY: Invalid token accepted')
except:
    print('✅ Invalid tokens rejected (expected)')
"

# Test 2: Rate limiting bypass
echo "2. Testing rate limiting bypass..."
python3 -c "
import requests
import time
from concurrent.futures import ThreadPoolExecutor

def make_request(i):
    try:
        resp = requests.get('http://localhost:8000/health', timeout=2)
        return resp.status_code
    except:
        return 0

# Rapid fire requests
with ThreadPoolExecutor(max_workers=50) as executor:
    futures = [executor.submit(make_request, i) for i in range(200)]
    results = [f.result() for f in futures]

status_codes = {}
for code in results:
    status_codes[code] = status_codes.get(code, 0) + 1

print(f'Status code distribution: {status_codes}')
if 429 in status_codes:
    print('✅ Rate limiting active')
else:
    print('❌ POTENTIAL ISSUE: No rate limiting detected')
"

# Test 3: Input validation bypass
echo "3. Testing input validation..."
python3 -c "
import requests
import json

# Test oversized content
try:
    resp = requests.post('http://localhost:8000/learn', 
                        json={'content': 'A' * 5000000},  # 5MB
                        timeout=5)
    if resp.status_code == 413 or resp.status_code == 400:
        print('✅ Oversized content rejected')
    else:
        print(f'❌ POTENTIAL ISSUE: Oversized content accepted ({resp.status_code})')
except requests.exceptions.RequestException:
    print('✅ Oversized content caused connection failure (good)')

# Test malicious payloads
malicious_payloads = [
    '<script>alert(1)</script>',
    '../../etc/passwd',
    '\x00\x01\x02\x03',  # Null bytes
    'SELECT * FROM users;',  # SQL injection attempt
]

for payload in malicious_payloads:
    try:
        resp = requests.post('http://localhost:8000/learn',
                            json={'content': payload},
                            timeout=5)
        print(f'Malicious payload response: {resp.status_code}')
    except:
        print('Malicious payload caused error (potentially good)')
"

# Test 4: TLS configuration
echo "4. Testing TLS configuration..."
if [ "${SUTRA_TLS_ENABLED:-false}" = "true" ]; then
    echo "Testing TLS cipher strength..."
    openssl s_client -connect localhost:50051 -cipher 'HIGH:!aNULL:!eNULL:!EXPORT:!DES:!RC4:!MD5:!PSK:!SRP:!CAMELLIA' -brief 2>/dev/null | head -5
    
    echo "Testing weak ciphers (should fail)..."
    if openssl s_client -connect localhost:50051 -cipher 'RC4' -brief 2>/dev/null; then
        echo "❌ VULNERABILITY: Weak cipher accepted"
    else
        echo "✅ Weak ciphers rejected"
    fi
else
    echo "⚠️  TLS not enabled - skipping TLS tests"
fi

# Test 5: Directory traversal  
echo "5. Testing directory traversal..."
curl -s "http://localhost:8000/../../etc/passwd" | head -1 | grep -q "root:" && {
    echo "❌ CRITICAL: Directory traversal vulnerability"
} || {
    echo "✅ Directory traversal blocked"
}

# Test 6: HTTP method tampering
echo "6. Testing HTTP method security..."
for method in DELETE PUT PATCH OPTIONS; do
    response=$(curl -s -X $method -w "%{http_code}" -o /dev/null "http://localhost:8000/health")
    if [ "$response" = "405" ]; then
        echo "✅ Method $method properly rejected"
    else
        echo "⚠️  Method $method returned: $response"
    fi
done

echo ""
echo "Penetration test complete. Review any vulnerabilities found above."
EOF

chmod +x /usr/local/bin/sutra-pentest
```

**Verification**:
```bash
# Run penetration test
/usr/local/bin/sutra-pentest

# Review results and address any vulnerabilities found
```

**Status**: [ ] Complete

---

## Final Security Checklist

### Critical Security Items (Must Complete)

- [ ] **1.1** Secure mode enabled (`SUTRA_SECURE_MODE=true`)
- [ ] **1.2** Strong authentication secret generated (≥64 characters)
- [ ] **1.3** Role-based access control configured
- [ ] **2.1** TLS encryption configured and tested
- [ ] **2.2** Firewall rules configured (block external access)  
- [ ] **2.3** Rate limiting configured and tested
- [ ] **3.1** Input validation hardening enabled
- [ ] **3.2** Secure deserialization limits configured
- [ ] **3.3** File system permissions secured
- [ ] **6.1** All security tests passing
- [ ] **6.2** Penetration test completed with no critical issues

### Important Security Items (Recommended)

- [ ] **4.1** Security event logging configured
- [ ] **4.2** Monitoring and alerting integrated  
- [ ] **4.3** Automated health checks enabled
- [ ] **5.1** Security configuration backup automated
- [ ] **5.2** Secret recovery procedures tested

### Optional Security Items (Enhanced Security)

- [ ] Certificate pinning implemented
- [ ] Network segmentation configured
- [ ] Intrusion detection system deployed
- [ ] Security information and event management (SIEM) integration
- [ ] Regular security assessments scheduled

---

## Post-Hardening Steps

### 1. Documentation Update
```bash
# Document your security configuration
cat > /etc/sutra/SECURITY_CONFIG.md << 'EOF'
# Sutra Security Configuration

**Deployment Date**: $(date -I)
**Hardening Completed By**: $(whoami)
**Security Level**: Production

## Configuration Summary
- Secure Mode: ENABLED
- Authentication: HMAC-SHA256  
- TLS Encryption: ENABLED
- Rate Limiting: ACTIVE
- Monitoring: CONFIGURED

## Key Files
- Auth Secret: /etc/sutra/auth-secret
- TLS Certificates: /etc/sutra/certs/
- Configuration: /etc/sutra/*.toml
- Logs: /var/log/sutra/

## Emergency Contacts
- Security Team: security@yourdomain.com
- Operations Team: ops@yourdomain.com

## Next Review Date
$(date -d "+90 days" -I)
EOF
```

### 2. Team Training
- [ ] Operations team trained on security procedures
- [ ] Incident response procedures documented and tested
- [ ] Security monitoring dashboards configured
- [ ] Backup and recovery procedures validated

### 3. Compliance Documentation
- [ ] Security configuration documented for compliance audit
- [ ] Risk assessment updated with hardening measures
- [ ] Security controls mapping completed
- [ ] Regular security review schedule established

---

**Document Version**: 1.0  
**Hardening Completion**: [ ] Complete  
**Review Date**: ___________  
**Approved By**: ___________