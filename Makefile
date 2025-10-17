.PHONY: help build push deploy clean test

REGISTRY ?= docker.io/sutraai
VERSION ?= latest
IMAGES = sutra-storage-server sutra-api sutra-hybrid sutra-control sutra-client

help:
	@echo "Sutra AI - Production Build and Deployment"
	@echo ""
	@echo "Available targets:"
	@echo "  build         - Build all Docker images"
	@echo "  push          - Push images to registry"
	@echo "  dev           - Start local development"
	@echo "  dev-down      - Stop local development"
	@echo "  k8s-deploy    - Deploy to Kubernetes"
	@echo "  clean         - Clean up images and containers"
	@echo ""

build:
	@echo "📦 Building all images..."
	./build-images.sh

tag:
	@for img in $(IMAGES); do \
		docker tag $$img:$(VERSION) $(REGISTRY)/$$img:$(VERSION); \
		docker tag $$img:$(VERSION) $(REGISTRY)/$$img:latest; \
	done

push: tag
	@for img in $(IMAGES); do \
		docker push $(REGISTRY)/$$img:$(VERSION); \
		docker push $(REGISTRY)/$$img:latest; \
	done

dev:
	@echo "🚀 Starting development environment..."
	docker compose up -d
	@docker compose ps

dev-down:
	docker compose down

dev-logs:
	docker compose logs -f

k8s-deploy:
	@echo "☸️  Deploying to Kubernetes..."
	kubectl apply -f k8s/00-namespace.yaml
	kubectl apply -f k8s/sutra-ai-deployment.yaml

k8s-delete:
	kubectl delete -f k8s/sutra-ai-deployment.yaml
	kubectl delete -f k8s/00-namespace.yaml

k8s-status:
	kubectl get all -n sutra-ai

clean:
	docker compose down -v
	@for img in $(IMAGES); do \
		docker rmi -f $$img:$(VERSION) || true; \
	done
