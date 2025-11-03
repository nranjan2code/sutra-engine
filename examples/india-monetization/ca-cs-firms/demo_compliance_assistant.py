"""
Sutra AI - CA/CS Compliance Knowledge Assistant Demo

This demo shows how Sutra can help CA/CS firms with:
1. Regulatory change tracking (temporal reasoning)
2. Contradiction detection between regulations
3. Client advisory automation
4. Institutional knowledge retention

Target: Medium CA/CS firms (20-100 CAs)
Pricing: ₹49,999/month
ROI: 33x (saves 2 hours/day × 20 CAs × ₹2,000/hour)
"""

import sys
import os
from datetime import datetime

# Add Sutra packages to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../../packages/sutra-core'))

from sutra_core.storage_client import StorageClient
from sutra_core.query_processor import QueryProcessor

class ComplianceAssistant:
    """AI-powered compliance assistant for CA/CS firms"""
    
    def __init__(self, storage_url="http://localhost:50051"):
        """Initialize with Sutra storage connection"""
        self.storage = StorageClient(storage_url)
        self.query_processor = QueryProcessor(self.storage)
        
    def learn_regulatory_change(self, regulation, old_value, new_value, effective_date):
        """Learn about regulatory changes with temporal context"""
        
        # Old regulation
        old_concept = self.storage.learn_concept(
            content=f"{regulation} {old_value}",
            metadata={
                "type": "Rule",
                "valid_until": effective_date,
                "source": "Government Gazette"
            }
        )
        
        # New regulation
        new_concept = self.storage.learn_concept(
            content=f"{regulation} {new_value}",
            metadata={
                "type": "Rule",
                "effective_from": effective_date,
                "source": "Government Gazette"
            }
        )
        
        # Temporal relationship
        self.storage.add_association(
            from_concept=old_concept,
            to_concept=new_concept,
            association_type="temporal",
            metadata={
                "relation": "superseded_by",
                "transition_date": effective_date
            }
        )
        
        print(f"✅ Learned: {regulation} changed on {effective_date}")
        return new_concept
    
    def learn_regulation(self, regulation_text, source, category="Compliance"):
        """Learn a regulation or compliance rule"""
        
        concept = self.storage.learn_concept(
            content=regulation_text,
            metadata={
                "type": "Rule",
                "category": category,
                "source": source,
                "learned_date": datetime.now().isoformat()
            }
        )
        
        print(f"✅ Learned: {regulation_text[:60]}...")
        return concept
    
    def detect_contradictions(self, topic):
        """Find contradicting regulations on a topic"""
        
        query = f"What are the contradicting requirements for {topic}?"
        results = self.query_processor.query_graph(
            query=query,
            max_paths=10
        )
        
        print(f"\n🔍 Checking for contradictions in: {topic}")
        print("=" * 80)
        
        if results:
            for idx, result in enumerate(results, 1):
                print(f"\nPath {idx} (Confidence: {result.confidence:.2f}):")
                for step in result.reasoning_path:
                    print(f"  → {step}")
        else:
            print("No contradictions found")
        
        return results
    
    def query_advisory(self, client_question):
        """Answer client advisory questions with reasoning"""
        
        results = self.query_processor.query_graph(
            query=client_question,
            max_paths=5
        )
        
        print(f"\n📋 Client Question: {client_question}")
        print("=" * 80)
        
        if results:
            # Show top result with reasoning
            top_result = results[0]
            print(f"\nRecommendation (Confidence: {top_result.confidence:.2f}):")
            print(f"{top_result.answer}\n")
            
            print("Reasoning:")
            for step in top_result.reasoning_path:
                print(f"  → {step}")
            
            # Show similar cases
            if len(results) > 1:
                print(f"\nSimilar cases:")
                for result in results[1:3]:
                    print(f"  • {result.answer} (Confidence: {result.confidence:.2f})")
        else:
            print("No relevant guidance found. Consider consulting primary sources.")
        
        return results
    
    def track_knowledge_source(self, topic, expert, insight):
        """Track institutional knowledge from senior partners"""
        
        expert_concept = self.storage.learn_concept(
            content=f"{expert} is an expert in {topic}",
            metadata={"type": "Entity", "role": "Expert"}
        )
        
        knowledge_concept = self.storage.learn_concept(
            content=insight,
            metadata={"type": "Definitional", "topic": topic}
        )
        
        # Associate knowledge with expert
        self.storage.add_association(
            from_concept=expert_concept,
            to_concept=knowledge_concept,
            association_type="causal",
            metadata={"relation": "contributed_knowledge"}
        )
        
        print(f"✅ Captured knowledge from {expert}: {insight[:50]}...")
        return knowledge_concept


def run_demo():
    """Run comprehensive demo for CA/CS firms"""
    
    print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                   Sutra AI - CA/CS Compliance Assistant                      ║
║                                                                              ║
║  Helping CA/CS firms with regulatory tracking, contradiction detection,     ║
║  client advisory automation, and institutional knowledge retention          ║
╚══════════════════════════════════════════════════════════════════════════════╝
    """)
    
    assistant = ComplianceAssistant()
    
    # ==================================================================================
    # Demo 1: Regulatory Change Tracking (Temporal Reasoning)
    # ==================================================================================
    print("\n" + "="*80)
    print("DEMO 1: Regulatory Change Tracking (Temporal Reasoning)")
    print("="*80)
    print("\nScenario: Finance Act 2024 amended Section 80IAC (Startup Tax Exemption)\n")
    
    assistant.learn_regulatory_change(
        regulation="Section 80IAC allows tax exemption for eligible startups for",
        old_value="3 consecutive years",
        new_value="5 consecutive years",
        effective_date="2024-04-01"
    )
    
    # Query temporal change
    print("\n💬 CA Query: 'What changed in startup tax exemption after April 2024?'")
    assistant.query_advisory("What changed in startup tax exemption after April 2024?")
    
    # ==================================================================================
    # Demo 2: Contradiction Detection
    # ==================================================================================
    print("\n\n" + "="*80)
    print("DEMO 2: Contradiction Detection Between Regulations")
    print("="*80)
    print("\nScenario: Different regulations have different board meeting requirements\n")
    
    assistant.learn_regulation(
        "Companies Act 2013 requires board meeting at least once every 120 days",
        source="Companies Act 2013, Section 173",
        category="Corporate Governance"
    )
    
    assistant.learn_regulation(
        "SEBI Listing Regulations require board meeting at least once every 90 days for listed companies",
        source="SEBI LODR Regulation 17(2)",
        category="Capital Markets"
    )
    
    assistant.learn_regulation(
        "RBI Master Directions require NBFC board meetings at least once every 60 days",
        source="RBI Master Direction - NBFC",
        category="Banking"
    )
    
    # Detect contradictions
    assistant.detect_contradictions("board meeting frequency for NBFC listed company")
    
    # ==================================================================================
    # Demo 3: Client Advisory Automation
    # ==================================================================================
    print("\n\n" + "="*80)
    print("DEMO 3: Client Advisory Automation")
    print("="*80)
    print("\nScenario: Client asks about startup tax exemption eligibility\n")
    
    # Learn eligibility criteria
    assistant.learn_regulation(
        "Startup qualifies for 80IAC exemption if annual turnover does not exceed 100 crore rupees",
        source="Finance Act 2024",
        category="Income Tax"
    )
    
    assistant.learn_regulation(
        "Startup qualifies for 80IAC exemption if incorporated less than 10 years ago",
        source="Finance Act 2024",
        category="Income Tax"
    )
    
    assistant.learn_regulation(
        "Startup does not qualify for 80IAC exemption if formed by splitting or reconstructing existing business",
        source="Finance Act 2024",
        category="Income Tax"
    )
    
    # Past case examples
    assistant.storage.learn_concept(
        content="Startup Alpha qualified for 80IAC exemption with turnover of 75 crore rupees, incorporated in 2022, original entity",
        metadata={"type": "Event", "category": "Case Study", "outcome": "Qualified"}
    )
    
    assistant.storage.learn_concept(
        content="Startup Beta did not qualify for 80IAC exemption because it was formed by demerger of existing company",
        metadata={"type": "Event", "category": "Case Study", "outcome": "Rejected"}
    )
    
    # Client query
    print("\n💬 Client: 'My startup has 85 crore turnover, incorporated 2023, original entity. Do I qualify for 80IAC?'")
    assistant.query_advisory("Does startup with 85 crore turnover incorporated in 2023 as original entity qualify for Section 80IAC tax exemption?")
    
    # ==================================================================================
    # Demo 4: Institutional Knowledge Retention
    # ==================================================================================
    print("\n\n" + "="*80)
    print("DEMO 4: Institutional Knowledge Retention (Partner Expertise)")
    print("="*80)
    print("\nScenario: Capture senior partner's M&A tax structuring expertise\n")
    
    assistant.track_knowledge_source(
        topic="M&A Tax Structuring",
        expert="Partner Rajesh Sharma",
        insight="For M&A deals, slump sale structure often saves 10-15% in taxes compared to asset sale, especially when buyer wants business as going concern"
    )
    
    assistant.track_knowledge_source(
        topic="M&A Tax Structuring",
        expert="Partner Rajesh Sharma",
        insight="Slump sale works best when target has valuable goodwill, as goodwill transfer in asset sale attracts capital gains"
    )
    
    # Junior CA query
    print("\n💬 Junior CA: 'How should we structure M&A deal to minimize tax for client?'")
    assistant.query_advisory("What is the best tax structure for M&A deal?")
    
    # ==================================================================================
    # Summary & Value Proposition
    # ==================================================================================
    print("\n\n" + "="*80)
    print("DEMO COMPLETE - VALUE PROPOSITION")
    print("="*80)
    
    print("""
✅ Regulatory Change Tracking
   → Automatically track amendments with temporal context
   → Save 2 hours/day of manual monitoring
   → Never miss important changes

✅ Contradiction Detection
   → Identify conflicting regulations automatically
   → Ensure complete compliance across all applicable laws
   → Reduce penalty risk

✅ Client Advisory Automation
   → Answer client questions with complete reasoning
   → Reference past cases and similar scenarios
   → Maintain consistency across team

✅ Knowledge Retention
   → Capture senior partner expertise
   → Make institutional knowledge searchable
   → Reduce dependency on individual partners

💰 ROI CALCULATION (20-CA Firm):
   Time saved: 2 hours/day × 20 CAs × 250 days = 10,000 hours/year
   Value: 10,000 hours × ₹2,000/hour = ₹2 crore/year
   Cost: ₹49,999/month × 12 = ₹6 lakhs/year
   
   ROI: ₹2 Cr saved ÷ ₹6 lakhs cost = 33x return

📈 PRICING:
   Basic (5 CAs): ₹15,999/month
   Professional (25 CAs): ₹49,999/month ⭐ RECOMMENDED
   Enterprise (Unlimited): ₹1,49,999/month

🎯 FREE TRIAL:
   2 months free for first 5 pilot firms
   No credit card required
   Cancel anytime

📞 NEXT STEPS:
   1. Schedule 15-minute demo: sales@sutra.ai
   2. Start free trial with your team
   3. See results in 2 weeks

═══════════════════════════════════════════════════════════════════════════════
   Sutra AI - Making compliance explainable, one query at a time
═══════════════════════════════════════════════════════════════════════════════
    """)


if __name__ == "__main__":
    try:
        run_demo()
    except ConnectionError:
        print("""
⚠️  ERROR: Cannot connect to Sutra storage server

Please ensure Sutra is running:
    ./sutra deploy

Or connect to remote server:
    export SUTRA_STORAGE_URL=http://your-server:50051
    python demo_compliance_assistant.py
        """)
    except Exception as e:
        print(f"\n❌ Demo failed: {e}")
        import traceback
        traceback.print_exc()
