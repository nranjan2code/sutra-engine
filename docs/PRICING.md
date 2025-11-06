# Sutra AI Pricing & Business Model

**Enterprise-grade domain reasoning at transparent, predictable pricing.**

## Pricing Tiers

| Edition | Monthly | Annual | Ideal For |
|---------|---------|--------|-----------|
| **Simple** | **FREE** | **FREE** | Development, testing, learning |
| **Community** | **$99** | **$990** (2 months free) | Small teams, MVPs, startups |
| **Enterprise** | **$999** | **Custom** (volume discounts) | Production, regulated industries |

## What's Included

### Simple Edition - FREE Forever

**Perfect for:**
- Individual developers
- Learning and experimentation
- Proof-of-concepts
- Non-production testing

**Includes:**
- ✅ All core features (reasoning, embeddings, NLG, semantic search)
- ✅ Control Center monitoring
- ✅ Bulk data ingestion
- ✅ Custom adapter support
- ✅ 7 Docker containers
- ✅ Single-node deployment

**Limits:**
- 10 learn requests/minute
- 50 reason requests/minute
- 100,000 max concepts
- 1GB max dataset size
- 2 concurrent ingestion workers
- 7 days audit log retention

**Support:**
- Community forums
- Documentation
- GitHub issues

**Deploy:**
```bash
sutra deploy
# No credit card required
# No license key needed
```

---

### Community Edition - $99/month

**Perfect for:**
- Small teams (2-10 people)
- MVPs and early-stage products
- Startups with <1M concepts
- B2B SaaS products

**Everything in Simple, plus:**
- ✅ **10× higher API limits** (100 learn/min, 500 reason/min)
- ✅ **10× more concepts** (1,000,000 max)
- ✅ **10× larger datasets** (10GB max)
- ✅ **2× ingestion speed** (4 concurrent workers)
- ✅ **30-day audit retention** (vs 7 days)
- ✅ **Email support** (48-hour response SLA)

**Same infrastructure as Simple:**
- 7 Docker containers
- Single-node deployment
- 4-8GB RAM recommended

**Annual Savings:**
- Monthly: $99 × 12 = $1,188
- Annual: $990 (save $198 = 2 months free)

**Deploy:**
```bash
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license-key"
sutra deploy
```

**Use Cases:**
- Customer support knowledge bases
- Internal company wikis
- Document Q&A systems
- Compliance tracking tools

---

### Enterprise Edition - $999/month

**Perfect for:**
- Production deployments
- Regulated industries (healthcare, finance, legal)
- Teams requiring 99.9% SLA
- >1M concepts
- Mission-critical applications

**Everything in Community, plus:**

**🚀 100× Scale:**
- 1000 learn requests/minute (100× Simple)
- 5000 reason requests/minute (100× Simple)
- 10,000,000 max concepts (100× Simple)
- Unlimited dataset size
- 16 concurrent ingestion workers (8× Simple)
- 365-day audit retention (compliance-ready)

**⚡ High Availability:**
- 3× embedding service replicas
- 3× NLG service replicas
- HAProxy load balancing
- Automatic failover (<3 second detection)
- Zero-downtime deployments
- **99.9% uptime SLA** (43.8 minutes/month max downtime)

**🌐 Grid Infrastructure:**
- Distributed processing across nodes
- Event-driven observability
- Multi-node orchestration
- Advanced monitoring & alerting
- Horizontal scalability

**🔒 Enterprise Security:**
- TLS encryption (mandatory)
- Authentication & authorization
- Network isolation
- Secret management
- Security audit logs

**🎯 Dedicated Support:**
- **4-hour response SLA** (business hours)
- Direct engineering escalation
- Slack/Teams integration
- Quarterly business reviews
- Professional services available

**Infrastructure:**
- 16 Docker containers
- Multi-node capable
- 20GB+ RAM recommended

**Annual Contract:**
- Volume discounts available
- Custom SLAs (e.g., 99.95%)
- Professional services credits
- Dedicated success manager

**Deploy:**
```bash
export SUTRA_EDITION="enterprise"
export SUTRA_LICENSE_KEY="your-license-key"
export SUTRA_SECURE_MODE="true"
sutra deploy
```

**Use Cases:**
- Hospital protocol systems
- Legal document analysis
- Financial compliance tracking
- Government knowledge bases
- Manufacturing quality control

---

## Feature Comparison Matrix

| Feature | Simple | Community | Enterprise |
|---------|--------|-----------|------------|
| **Deployment** | | | |
| Containers | 7 | 7 | 16 |
| Memory requirement | 4GB | 8GB | 20GB |
| Setup time | 60 sec | 60 sec | 3 min |
| | | | |
| **Core Features** | | | |
| Graph reasoning | ✅ | ✅ | ✅ |
| Semantic embeddings | ✅ | ✅ | ✅ |
| Semantic reasoning | ✅ | ✅ | ✅ |
| NLG (template + LLM) | ✅ | ✅ | ✅ |
| Control Center | ✅ | ✅ | ✅ |
| Bulk ingestion | ✅ | ✅ | ✅ |
| Custom adapters | ✅ | ✅ | ✅ |
| | | | |
| **API Limits** | | | |
| Learn requests/min | 10 | 100 | **1000** |
| Reason requests/min | 50 | 500 | **5000** |
| Max concepts | 100K | 1M | **10M** |
| Max dataset size | 1GB | 10GB | **Unlimited** |
| Ingestion workers | 2 | 4 | **16** |
| Audit retention | 7 days | 30 days | **365 days** |
| | | | |
| **Availability** | | | |
| Uptime SLA | None | Best effort | **99.9%** |
| Embedding replicas | 1 | 1 | **3** |
| NLG replicas | 1 | 1 | **3** |
| Load balancing | ❌ | ❌ | **✅** |
| Auto-failover | ❌ | ❌ | **✅** |
| Zero-downtime deploys | ❌ | ❌ | **✅** |
| | | | |
| **Enterprise Features** | | | |
| Grid orchestration | ❌ | ❌ | **✅** |
| Distributed processing | ❌ | ❌ | **✅** |
| Event observability | ❌ | ❌ | **✅** |
| Multi-node support | ❌ | ❌ | **✅** |
| Mandatory security | ❌ | ❌ | **✅** |
| | | | |
| **Support** | | | |
| Community forums | ✅ | ✅ | ✅ |
| Email support | ❌ | ✅ (48h) | **✅ (4h)** |
| Slack/Teams | ❌ | ❌ | **✅** |
| Engineering escalation | ❌ | ❌ | **✅** |
| Quarterly reviews | ❌ | ❌ | **✅** |
| Professional services | ❌ | ❌ | **✅** |

---

## Professional Services

Available for **Enterprise customers** only:

### Custom Adapter Development
**Starting at $5,000**
- Domain-specific data adapters
- Integration with your systems (ERP, CRM, databases)
- Custom ingestion pipelines
- Testing & documentation included

### Deployment Assistance
**Starting at $2,500**
- Production architecture review
- Performance tuning
- Security hardening
- Load testing & optimization

### Training & Onboarding
**$1,500/day (on-site) or $1,000/day (remote)**
- Team training sessions
- Best practices workshops
- Custom use case development
- Ongoing Q&A sessions

### Dedicated Support Contracts
**Custom pricing**
- 1-hour response SLA
- 24/7 emergency support
- Dedicated Slack channel
- Monthly architecture reviews

**Contact:** professional-services@sutra.ai

---

## Upgrade Path

### From Simple to Community

**When to upgrade:**
- Approaching 100K concepts
- Need >10 requests/minute
- Require email support
- Moving to production (non-critical)

**How to upgrade:**
```bash
# 1. Stop services (no data loss!)
sutra stop

# 2. Get license from https://sutra.ai/pricing

# 3. Set edition & restart
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license-key"
sutra deploy
```

**Zero downtime:** Data persists across upgrades

---

### From Community to Enterprise

**When to upgrade:**
- Approaching 1M concepts
- Need high availability
- Require 99.9% SLA
- Moving to critical production
- Regulated industry requirements

**How to upgrade:**
```bash
# 1. Stop services (no data loss!)
sutra stop

# 2. Get Enterprise license

# 3. Set edition + security
export SUTRA_EDITION="enterprise"
export SUTRA_LICENSE_KEY="your-enterprise-license"
export SUTRA_SECURE_MODE="true"
sutra deploy
```

**Upgrade includes:** Architecture review + deployment assistance

---

## Cost Comparison

### vs. OpenAI API (GPT-4)

| Metric | OpenAI GPT-4 | Sutra Enterprise |
|--------|--------------|------------------|
| **Setup** | $0 | $999/mo |
| **Per query** | $0.03-0.10 | $0 |
| **1M queries/mo** | $30,000-100,000 | $999 |
| **Data privacy** | Sent to OpenAI | Self-hosted |
| **Audit trails** | None | Complete |
| **Domain learning** | $10K-100K fine-tuning | Real-time |

**Break-even:** 33,000 queries/month (Simple = 43K queries/month at max rate)

---

### vs. Elasticsearch + Vector DB

| Component | Elasticsearch Stack | Sutra Enterprise |
|-----------|---------------------|------------------|
| **Search** | $200-500/mo | Included |
| **Vector DB** | $300-800/mo | Included |
| **Embedding API** | $50-200/mo | Included |
| **Development** | 3-6 months | 1 day |
| **Maintenance** | Ongoing | Managed |
| **Total** | $550-1500/mo + dev | $999/mo |

**Savings:** 3-6 months development time + ongoing maintenance

---

### vs. Fine-tuned LLMs

| Approach | Cost | Time | Data Requirements |
|----------|------|------|-------------------|
| **Fine-tuning GPT** | $10K-100K | Weeks | Large labeled dataset |
| **Training custom LLM** | $100K-1M | Months | Massive dataset |
| **Sutra Enterprise** | $999/mo | Hours | Your existing docs |

**Advantage:** Real-time learning vs. expensive retraining

---

## ROI Calculator

### Example: Customer Support Knowledge Base

**Scenario:** 10-person support team, 100K support tickets/year

**Without Sutra:**
- Support time: 100K tickets × 5 min = 500K minutes = 8,333 hours
- Cost: 8,333 hours × $50/hour = $416,650/year

**With Sutra (Community):**
- Support time: 100K tickets × 2 min = 200K minutes = 3,333 hours (60% faster)
- Cost: 3,333 hours × $50/hour = $166,650/year
- Sutra cost: $99 × 12 = $1,188/year
- **Total savings: $248,812/year (60% reduction)**

**ROI:** 20,844% in year 1

---

## Frequently Asked Questions

### Licensing

**Q: Can I try Enterprise before buying?**  
A: Yes! Contact sales@sutra.ai for a 30-day Enterprise trial license.

**Q: What happens if my license expires?**  
A: 7-day grace period. Services continue running but won't restart. Renew at https://sutra.ai/pricing

**Q: Can I downgrade?**  
A: Yes. Data is preserved. HA/Grid features are disabled when downgrading from Enterprise.

**Q: Is there a non-profit discount?**  
A: Yes. 50% off Community and Enterprise for registered non-profits. Contact sales@sutra.ai

**Q: Do you offer academic pricing?**  
A: Yes. Free Community edition for universities and research institutions. Contact academic@sutra.ai

---

### Deployment

**Q: Can I run Sutra in the cloud?**  
A: Yes. Works on AWS, GCP, Azure, or any cloud with Docker support.

**Q: Do you offer managed hosting?**  
A: Yes (Enterprise only). Contact sales@sutra.ai for managed deployment options.

**Q: What if I need more than 10M concepts?**  
A: Contact sales@sutra.ai for custom Enterprise plans with unlimited concepts.

**Q: Can I scale horizontally?**  
A: Yes (Enterprise only). Grid infrastructure supports multi-node distribution.

---

### Support

**Q: What's included in "Professional Services"?**  
A: Custom adapter development, deployment assistance, training, and dedicated support. Enterprise only.

**Q: Can I get faster than 4-hour support?**  
A: Yes. Custom SLA contracts available with 1-hour response time. Contact sales@sutra.ai

**Q: Do you have a partner program?**  
A: Yes. Consultancies and system integrators get partner pricing. Contact partners@sutra.ai

---

## Get Started

### Simple Edition
**No credit card required. No license needed.**

```bash
git clone https://github.com/yourusername/sutra-models
cd sutra-models
sutra deploy
```

### Community Edition
**$99/month or $990/year (save $198)**

1. Sign up at https://sutra.ai/pricing
2. Receive license key via email
3. Deploy with license key

### Enterprise Edition
**$999/month or custom annual contract**

1. Book demo: https://sutra.ai/demo
2. Get custom quote
3. Receive Enterprise license + onboarding

---

## Contact

- **Sales:** sales@sutra.ai
- **Support:** support@sutra.ai
- **Partners:** partners@sutra.ai
- **Security:** security@sutra.ai

**Website:** https://sutra.ai  
**Documentation:** https://docs.sutra.ai  
**Community:** https://community.sutra.ai
