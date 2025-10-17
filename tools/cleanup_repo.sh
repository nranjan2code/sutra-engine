#!/usr/bin/env bash
set -euo pipefail

# Cleanup and consolidate Dockerfiles, Kubernetes manifests, and docs
# Non-destructive: originals are moved to *archive* or suffixed with .legacy.<timestamp>

TS="$(date +%Y%m%d%H%M%S)"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p k8s/archive docs/archive

# 1) Promote optimized Dockerfiles for api, hybrid, control
for PKG in packages/sutra-api packages/sutra-hybrid packages/sutra-control; do
  if [[ -f "$PKG/Dockerfile.optimized" ]]; then
    if [[ -f "$PKG/Dockerfile" ]]; then
      echo "Backing up $PKG/Dockerfile -> $PKG/Dockerfile.legacy.$TS"
      mv "$PKG/Dockerfile" "$PKG/Dockerfile.legacy.$TS"
    fi
    echo "Promoting $PKG/Dockerfile.optimized -> $PKG/Dockerfile"
    mv "$PKG/Dockerfile.optimized" "$PKG/Dockerfile"
  fi
done
# Note: sutra-client and sutra-storage Dockerfiles are left as-is

# 2) Keep only selected Kubernetes manifests, archive the rest
# Keep: 00-namespace.yaml, hpa.yaml, sutra-ai-deployment.yaml
if compgen -G "k8s/*.yaml" > /dev/null; then
  for F in k8s/*.yaml; do
    BNAME="$(basename "$F")"
    case "$BNAME" in
      00-namespace.yaml|hpa.yaml|sutra-ai-deployment.yaml)
        echo "Keeping k8s/$BNAME"
        ;;
      *)
        echo "Archiving k8s/$BNAME -> k8s/archive/$BNAME"
        mv "$F" "k8s/archive/$BNAME"
        ;;
    esac
  done
fi

# 3) Retain deployment automation scripts
# Keep deploy-optimized.sh as-is. Optionally archive older deployment docs.
for DOC in DEPLOYMENT.md README_PRODUCTION.md PRODUCTION_DEPLOYMENT_GUIDE.md; do
  if [[ -f "$DOC" ]]; then
    echo "Archiving $DOC -> docs/archive/$DOC"
    mv "$DOC" "docs/archive/$DOC"
  fi
done

# 4) Final summary
echo
echo "Cleanup complete. Summary:"
echo " - Promoted optimized Dockerfiles for api/hybrid/control (backed up originals)."
echo " - Archived non-selected k8s manifests to k8s/archive/."
echo " - Archived legacy deployment docs to docs/archive/."
echo
echo "Next steps:"
echo " - Review changes: git status && git diff -- k8s docs packages/*/Dockerfile*"
echo " - Build and deploy with: bash deploy-optimized.sh"
