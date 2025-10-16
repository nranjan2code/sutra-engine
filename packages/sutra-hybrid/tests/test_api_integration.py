"""Integration tests for Sutra AI API.

Tests both OpenAI-compatible and custom Sutra endpoints.
"""

import shutil
import tempfile
from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from sutra_hybrid import SutraAI
from sutra_hybrid.api import create_app


@pytest.fixture
def temp_storage():
    """Create temporary storage directory."""
    temp_dir = tempfile.mkdtemp()
    yield temp_dir
    shutil.rmtree(temp_dir)


@pytest.fixture
def ai_instance(temp_storage):
    """Create SutraAI instance for testing."""
    return SutraAI(storage_path=temp_storage)


@pytest.fixture
def client(ai_instance):
    """Create FastAPI test client."""
    app = create_app(ai_instance=ai_instance)
    return TestClient(app)


@pytest.fixture
def learned_ai(ai_instance):
    """AI instance with some pre-learned knowledge."""
    ai_instance.learn("Python is a programming language")
    ai_instance.learn("Machine learning is a subset of AI")
    ai_instance.learn("Neural networks are used in deep learning")
    return ai_instance


# ============================================================================
# Root and Health Endpoints
# ============================================================================


def test_root_endpoint(client):
    """Test root endpoint."""
    response = client.get("/")
    assert response.status_code == 200
    data = response.json()
    assert data["message"] == "Sutra AI API"
    assert "openapi_endpoints" in data
    assert "sutra_endpoints" in data


def test_ping_endpoint(client):
    """Test ping endpoint."""
    response = client.get("/ping")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_health_endpoint(client):
    """Test Sutra health endpoint."""
    response = client.get("/sutra/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["version"] == "1.0.0"
    assert "uptime_seconds" in data
    assert "total_concepts" in data
    assert "total_associations" in data


# ============================================================================
# OpenAI-Compatible Endpoints
# ============================================================================


def test_list_models(client):
    """Test list models endpoint."""
    response = client.get("/v1/models")
    assert response.status_code == 200
    data = response.json()
    assert data["object"] == "list"
    assert len(data["data"]) > 0
    assert data["data"][0]["id"] == "sutra-1"


def test_retrieve_model(client):
    """Test retrieve model endpoint."""
    response = client.get("/v1/models/sutra-1")
    assert response.status_code == 200
    data = response.json()
    assert data["id"] == "sutra-1"
    assert data["object"] == "model"


def test_retrieve_model_not_found(client):
    """Test retrieve non-existent model."""
    response = client.get("/v1/models/nonexistent")
    assert response.status_code == 404


def test_chat_completions_basic(client, learned_ai):
    """Test basic chat completion."""
    # Update client with learned AI
    from sutra_hybrid.api import openai_endpoints, sutra_endpoints

    openai_endpoints.set_ai_instance(learned_ai)
    sutra_endpoints.set_ai_instance(learned_ai)

    response = client.post(
        "/v1/chat/completions",
        json={
            "model": "sutra-1",
            "messages": [{"role": "user", "content": "What is Python?"}],
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert data["object"] == "chat.completion"
    assert len(data["choices"]) == 1
    assert data["choices"][0]["message"]["role"] == "assistant"
    assert len(data["choices"][0]["message"]["content"]) > 0
    assert "usage" in data


def test_chat_completions_with_system_message(client):
    """Test chat completion with system message."""
    response = client.post(
        "/v1/chat/completions",
        json={
            "model": "sutra-1",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant. Python is a programming language.",
                },
                {"role": "user", "content": "What is Python?"},
            ],
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert data["choices"][0]["message"]["role"] == "assistant"


def test_chat_completions_no_user_message(client):
    """Test chat completion without user message."""
    response = client.post(
        "/v1/chat/completions",
        json={
            "model": "sutra-1",
            "messages": [{"role": "system", "content": "You are a helpful assistant."}],
        },
    )
    assert response.status_code == 400


# ============================================================================
# Sutra Learning Endpoints
# ============================================================================


def test_learn_endpoint(client):
    """Test learning endpoint."""
    response = client.post(
        "/sutra/learn",
        json={"text": "Python is a programming language"},
    )
    assert response.status_code == 200
    data = response.json()
    assert data["success"] is True
    assert data["concepts_learned"] > 0
    assert "message" in data


def test_learn_empty_text(client):
    """Test learning with empty text."""
    response = client.post(
        "/sutra/learn",
        json={"text": ""},
    )
    # Should still succeed but learn nothing
    assert response.status_code in [200, 500]


# ============================================================================
# Sutra Query Endpoints
# ============================================================================


def test_query_endpoint(client, learned_ai):
    """Test query endpoint."""
    # Update client with learned AI
    from sutra_hybrid.api import openai_endpoints, sutra_endpoints

    openai_endpoints.set_ai_instance(learned_ai)
    sutra_endpoints.set_ai_instance(learned_ai)

    response = client.post(
        "/sutra/query",
        json={
            "query": "What is Python?",
            "semantic_boost": True,
            "max_depth": 3,
            "max_paths": 5,
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert "answer" in data
    assert "confidence" in data
    assert "confidence_breakdown" in data
    assert "reasoning_paths" in data
    assert "explanation" in data
    assert "timestamp" in data


def test_query_without_semantic_boost(client, learned_ai):
    """Test query without semantic boost."""
    from sutra_hybrid.api import openai_endpoints, sutra_endpoints

    openai_endpoints.set_ai_instance(learned_ai)
    sutra_endpoints.set_ai_instance(learned_ai)

    response = client.post(
        "/sutra/query",
        json={
            "query": "What is Python?",
            "semantic_boost": False,
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert data["confidence_breakdown"]["semantic_confidence"] == 0.0


def test_query_invalid_depth(client):
    """Test query with invalid depth."""
    response = client.post(
        "/sutra/query",
        json={
            "query": "What is Python?",
            "max_depth": 100,  # Too large
        },
    )
    assert response.status_code == 422


# ============================================================================
# Multi-Strategy Endpoints
# ============================================================================


def test_multi_strategy_endpoint(client, learned_ai):
    """Test multi-strategy comparison endpoint."""
    from sutra_hybrid.api import openai_endpoints, sutra_endpoints

    openai_endpoints.set_ai_instance(learned_ai)
    sutra_endpoints.set_ai_instance(learned_ai)

    response = client.post(
        "/sutra/multi-strategy",
        json={
            "query": "What is Python?",
            "max_depth": 3,
            "max_paths": 5,
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert "query" in data
    assert "strategies" in data
    assert len(data["strategies"]) == 2  # Graph-only and semantic-enhanced
    assert "agreement_score" in data
    assert "recommended_answer" in data
    assert "explanation" in data


# ============================================================================
# Statistics and Audit Endpoints
# ============================================================================


def test_stats_endpoint(client, learned_ai):
    """Test statistics endpoint."""
    from sutra_hybrid.api import openai_endpoints, sutra_endpoints

    openai_endpoints.set_ai_instance(learned_ai)
    sutra_endpoints.set_ai_instance(learned_ai)

    response = client.get("/sutra/stats?top_n=5")
    assert response.status_code == 200
    data = response.json()
    assert "total_concepts" in data
    assert "total_associations" in data
    assert "avg_concept_strength" in data
    assert "top_concepts" in data
    assert len(data["top_concepts"]) <= 5


def test_audit_endpoint(client, learned_ai):
    """Test audit log endpoint."""
    from sutra_hybrid.api import openai_endpoints, sutra_endpoints

    openai_endpoints.set_ai_instance(learned_ai)
    sutra_endpoints.set_ai_instance(learned_ai)

    # Perform some operations first
    client.post("/sutra/learn", json={"text": "Test learning"})
    client.post("/sutra/query", json={"query": "What is Python?"})

    response = client.get("/sutra/audit?limit=10")
    assert response.status_code == 200
    data = response.json()
    assert "total_entries" in data
    assert "entries" in data
    assert len(data["entries"]) <= 10


# ============================================================================
# Error Handling Tests
# ============================================================================


def test_invalid_json(client):
    """Test handling of invalid JSON."""
    response = client.post(
        "/sutra/learn",
        data="not json",
        headers={"Content-Type": "application/json"},
    )
    assert response.status_code == 422


def test_missing_required_field(client):
    """Test missing required field."""
    response = client.post(
        "/sutra/query",
        json={},  # Missing 'query' field
    )
    assert response.status_code == 422


# ============================================================================
# End-to-End Workflow Tests
# ============================================================================


def test_full_workflow(client):
    """Test complete workflow: learn -> query -> multi-strategy."""
    # 1. Learn some facts
    learn_response = client.post(
        "/sutra/learn",
        json={"text": "Rust is a systems programming language focused on safety"},
    )
    assert learn_response.status_code == 200

    # 2. Query the learned information
    query_response = client.post(
        "/sutra/query",
        json={"query": "What is Rust?"},
    )
    assert query_response.status_code == 200
    query_data = query_response.json()
    assert len(query_data["answer"]) > 0

    # 3. Compare strategies
    multi_response = client.post(
        "/sutra/multi-strategy",
        json={"query": "What is Rust?"},
    )
    assert multi_response.status_code == 200
    multi_data = multi_response.json()
    assert len(multi_data["strategies"]) == 2

    # 4. Check statistics
    stats_response = client.get("/sutra/stats")
    assert stats_response.status_code == 200
    stats_data = stats_response.json()
    assert stats_data["total_concepts"] > 0


def test_openai_compatibility_workflow(client):
    """Test OpenAI-compatible workflow."""
    # Learn via system message in chat
    response1 = client.post(
        "/v1/chat/completions",
        json={
            "model": "sutra-1",
            "messages": [
                {
                    "role": "system",
                    "content": "FastAPI is a modern web framework for Python",
                },
                {"role": "user", "content": "What is FastAPI?"},
            ],
        },
    )
    assert response1.status_code == 200

    # Query again without system message
    response2 = client.post(
        "/v1/chat/completions",
        json={
            "model": "sutra-1",
            "messages": [{"role": "user", "content": "Tell me about FastAPI"}],
        },
    )
    assert response2.status_code == 200


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
