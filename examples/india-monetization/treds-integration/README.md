# TReDS Platform Integration - Credit Risk API

**Target Market:** 3 TReDS Platforms (Invoice Mart, M1xchange, RXIL)  
**Market Size:** ₹10,000+ Cr invoice discounting market  
**Year 1 ARR Potential:** ₹60 lakhs (3 platforms)  
**Time to Revenue:** 6-8 weeks

---

## What is TReDS?

**Trade Receivables Discounting System (TReDS)** is a government-mandated platform for MSME invoice financing.

### Key Facts
- **3 platforms:** Invoice Mart, M1xchange, RXIL (RBI approved)
- **Volume:** ₹10,000+ Cr annual discounting
- **Mandate:** Large companies must onboard MSMEs on TReDS for invoice financing
- **Problem:** Platforms need real-time MSME credit risk assessment

---

## The Opportunity

### Current TReDS Process
1. MSME uploads invoice to TReDS platform
2. Platform verifies invoice authenticity (1-2 days)
3. **Manual credit check** of MSME (2-3 days) ← PAIN POINT
4. Invoice listed for discounting
5. Financiers bid on invoice
6. MSME receives payment (typically 80-90% of invoice value)

**Total time:** 5-7 days from upload to payment

### Sutra Solution
Replace manual credit check with AI-powered real-time assessment:

```
MSME uploads invoice
    ↓
Sutra API call (< 1 minute)
    ↓
Credit risk score + reasoning returned
    ↓
Invoice listed same day
    ↓
Payment in 24-48 hours
```

**Time saved:** 3-5 days per invoice → faster liquidity for MSMEs

---

## Product: MSME Credit Risk API

### Features

#### 1. Real-Time Creditworthiness Scoring
```python
POST /api/v1/credit-risk-assessment
{
    "udyam_number": "UDYAM-XX-00-1234567",
    "invoice_amount": 500000,
    "buyer_pan": "ABCDE1234F"
}

Response:
{
    "risk_score": 72,  # 0-100 scale
    "risk_category": "MEDIUM",
    "approval_recommendation": "APPROVE_WITH_LIMITS",
    "max_financing_amount": 400000,  # 80% of invoice
    "reasoning_paths": [
        {
            "path_id": 1,
            "confidence": 0.85,
            "reasoning": [
                "MSME has 3 years operating history (positive)",
                "No payment defaults in last 12 months (positive)",
                "Buyer is AAA-rated corporate (positive)",
                "Invoice amount is 2x average historical (caution)"
            ]
        }
    ],
    "temporal_insights": {
        "revenue_trend": "Growing 20% YoY",
        "payment_history": "100% on-time last 6 months",
        "business_age": "3.5 years"
    },
    "causal_factors": {
        "positive": [
            "Repeat buyer relationship (5 previous invoices, all paid)",
            "Manufacturing sector (lower default risk)",
            "Udyam registration current and verified"
        ],
        "negative": [
            "Single buyer concentration (80% of revenue)",
            "No export history (limited diversification)"
        ]
    },
    "audit_trail": {
        "assessment_id": "ASS-20251103-001234",
        "timestamp": "2025-11-03T14:30:00Z",
        "data_sources": ["Udyam", "GST", "Bank statement"],
        "compliance": "RBI_MSME_LENDING_GUIDELINES"
    }
}
```

#### 2. Temporal Analysis (Evolution Tracking)
- Track MSME business trajectory over time
- Detect improving/deteriorating trends
- Flag sudden changes (revenue spike/drop)

#### 3. Causal Understanding
- Identify root causes of risk factors
- Multi-hop reasoning (buyer risk → MSME risk)
- Preventive insights ("What could cause default?")

#### 4. Complete Audit Trail
- Every assessment fully explainable
- Regulatory compliance (RBI guidelines)
- Dispute resolution support

---

## Pricing Model

### Per-Assessment Pricing (Recommended)
```
Basic Assessment: ₹50 per API call
  - Risk score (0-100)
  - Approval recommendation
  - Basic reasoning (3 paths)

Standard Assessment: ₹100 per API call
  - All Basic features
  - Temporal insights (trends)
  - 5 reasoning paths
  - Audit trail

Premium Assessment: ₹200 per API call
  - All Standard features
  - Causal analysis (root causes)
  - 10 reasoning paths
  - Detailed audit trail with data sources
  - SLA: <30 second response time
```

### Volume Discounts
```
0-10,000 assessments/month: Full price
10,001-50,000: 10% discount
50,001-100,000: 20% discount
100,000+: 25% discount + dedicated support
```

### Alternative: Subscription Model
```
Starter: ₹2 lakhs/month
  - 5,000 assessments included
  - ₹40 per additional assessment
  - Target: Smaller TReDS platform

Professional: ₹5 lakhs/month
  - 15,000 assessments included
  - ₹35 per additional assessment
  - Priority support

Enterprise: ₹15 lakhs/month
  - 50,000 assessments included
  - ₹30 per additional assessment
  - Dedicated account manager
  - Custom integrations
```

---

## Revenue Projections

### Conservative Estimate (Per Platform)

**Invoice Mart** (largest platform):
- Monthly volume: 15,000 invoices
- Sutra adoption: 50% (7,500 invoices)
- Pricing: ₹100/assessment (Standard)
- **Monthly revenue:** ₹7.5 lakhs
- **Annual revenue:** ₹90 lakhs

**M1xchange** (mid-size):
- Monthly volume: 10,000 invoices
- Adoption: 50% (5,000)
- Pricing: ₹100/assessment
- **Monthly revenue:** ₹5 lakhs
- **Annual revenue:** ₹60 lakhs

**RXIL** (smaller, newer):
- Monthly volume: 5,000 invoices
- Adoption: 30% (1,500)
- Pricing: ₹100/assessment
- **Monthly revenue:** ₹1.5 lakhs
- **Annual revenue:** ₹18 lakhs

**Total TReDS Market:**
- **Year 1:** ₹60-90 lakhs ARR (conservative)
- **Year 2:** ₹1.5-2 Cr ARR (full adoption)
- **Year 3:** ₹3-4 Cr ARR (volume growth + price increase)

---

## Go-To-Market Strategy

### Phase 1: Platform Selection (Week 1-2)

**Target Order:**
1. **RXIL** (smallest, easier to penetrate)
   - Reason: Newer platform, more agile
   - Decision makers: Smaller team
   - POC: 1,000 free assessments

2. **M1xchange** (mid-size, MSM focus)
   - Reason: Strong MSME focus aligns with Sutra
   - POC: 2,000 free assessments

3. **Invoice Mart** (largest, enterprise)
   - Reason: Largest volume, but slower decision
   - POC: 5,000 free assessments

### Phase 2: Outreach (Week 3-4)

**Email Template:**
```
Subject: Reduce MSME credit assessment time from 3 days to <1 minute

Dear [Platform Head],

TReDS platforms are mandated to verify MSME creditworthiness before 
listing invoices. This currently takes 2-3 days of manual work.

Sutra AI's Credit Risk API can:
✅ Assess MSME credit risk in <1 minute (vs 3 days)
✅ Provide explainable risk scores with complete audit trails
✅ Track MSME business evolution over time (improving/deteriorating)
✅ Identify causal risk factors (not just correlations)

We'd like to offer [Platform Name] 1,000 free credit assessments 
as a pilot. If satisfied, pricing is ₹100 per assessment.

Can we schedule a 20-minute demo this week?

Best regards,
[Name]
Sutra AI
```

**LinkedIn Approach:**
- Connect with: Product heads, Risk heads, CTOs
- Message: Same pitch, shorter (3 sentences + demo CTA)

### Phase 3: Demo & POC (Week 5-8)

**Demo Script (20 minutes):**

1. **Problem Statement (3 min)**
   - Current: 3-day manual credit checks delay invoice financing
   - Impact: MSMEs wait 5-7 days for liquidity
   - Cost: Manual team of 10-20 people per platform

2. **Live Demo (10 min)**
   - Show: Real MSME Udyam number input
   - Result: Risk score in <1 minute with reasoning
   - Highlight: Temporal trends, causal factors, audit trail

3. **ROI Calculation (3 min)**
   - Time saved: 3 days → 1 minute = 4,320x faster
   - Cost saved: Reduce manual team from 20 → 5 people
   - MSME benefit: Faster liquidity = better working capital

4. **POC Proposal (4 min)**
   - Offer: 1,000 free assessments (₹1 lakh value)
   - Duration: 1 month
   - Success criteria: 90%+ accuracy, <30 sec response time
   - Next: Paid deployment at ₹100/assessment

**POC Technical Setup:**
- API integration: 1 week (platform tech team + Sutra)
- Testing: 1 week (parallel run with manual process)
- Validation: 2 weeks (compare Sutra vs manual scores)

### Phase 4: Paid Deployment (Week 9-12)

**Pricing Negotiation:**
- Start: ₹100/assessment (Standard)
- Volume discount: 10% off if >10K assessments/month
- Annual contract: 15% discount for 1-year commitment
- **Expected close:** ₹85-90/assessment for large platforms

**Contract Terms:**
- Payment: Monthly in arrears (after assessments delivered)
- SLA: 99.5% uptime, <30 sec response time
- Data privacy: All data encrypted, no storage beyond 30 days
- Compliance: RBI MSME lending guidelines adherence

---

## Technical Implementation

### API Integration (Platform Side)

```python
# TReDS platform integration example

import requests

def assess_msme_credit(udyam_number, invoice_amount, buyer_pan):
    """
    Call Sutra Credit Risk API from TReDS platform
    """
    
    url = "https://api.sutra.ai/v1/credit-risk-assessment"
    headers = {
        "Authorization": "Bearer YOUR_API_KEY",
        "Content-Type": "application/json"
    }
    
    payload = {
        "udyam_number": udyam_number,
        "invoice_amount": invoice_amount,
        "buyer_pan": buyer_pan,
        "assessment_type": "STANDARD"  # or BASIC, PREMIUM
    }
    
    response = requests.post(url, json=payload, headers=headers, timeout=30)
    
    if response.status_code == 200:
        assessment = response.json()
        
        # Extract key fields for TReDS workflow
        risk_score = assessment["risk_score"]
        recommendation = assessment["approval_recommendation"]
        max_financing = assessment["max_financing_amount"]
        
        # Log audit trail for compliance
        audit_id = assessment["audit_trail"]["assessment_id"]
        
        return {
            "approved": recommendation in ["APPROVE", "APPROVE_WITH_LIMITS"],
            "risk_score": risk_score,
            "max_amount": max_financing,
            "audit_id": audit_id,
            "reasoning": assessment["reasoning_paths"][0]["reasoning"]
        }
    else:
        # Fallback to manual process
        raise Exception(f"API error: {response.status_code}")
```

**Integration Time:** 1-2 weeks (platform tech team + Sutra support)

---

## Success Metrics

### POC Phase (Month 1-2)
- ✅ 1,000 assessments completed
- ✅ 90%+ accuracy vs manual review
- ✅ <30 second average response time
- ✅ Zero compliance violations
- ✅ Platform satisfaction: 4.5/5

### Paid Deployment (Month 3-6)
- ✅ 5,000+ assessments/month per platform
- ✅ ₹5-10 lakhs MRR
- ✅ 95%+ API uptime
- ✅ <10% dispute rate (risk score challenged)

### Scale (Month 7-12)
- ✅ All 3 platforms live
- ✅ 15,000+ assessments/month total
- ✅ ₹15 lakhs MRR (₹1.8 Cr ARR)
- ✅ Expand to banks (use TReDS as case study)

---

## Competitive Advantages

### vs. Credit Bureaus (CIBIL, Experian)
- **CIBIL:** Consumer credit only, no MSME insights
- **Sutra:** MSME-specific with business evolution tracking

### vs. Manual Assessment
- **Manual:** 3 days, subjective, no audit trail
- **Sutra:** <1 minute, objective, complete explainability

### vs. Rule-Based Systems
- **Rules:** Static thresholds, no learning
- **Sutra:** AI learns patterns, temporal + causal understanding

---

## Next Steps

### This Week
1. Identify contact details for 3 TReDS platform heads
2. Draft outreach emails (personalized per platform)
3. Build API demo environment

### Next Week
1. Send outreach emails
2. Schedule demos (target: 2/3 platforms)
3. Prepare POC proposal documents

### Month 1
1. Conduct 2-3 platform demos
2. Sign 1 POC agreement
3. Complete API integration

### Month 2-3
1. Execute POC successfully
2. Convert to paid contract
3. Generate first TReDS revenue

**Budget Required:** ₹50,000 (API infrastructure + demo setup)  
**Expected First Revenue:** Month 3 (₹1.5-3 lakhs)
