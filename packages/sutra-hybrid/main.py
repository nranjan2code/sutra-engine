#!/usr/bin/env python3
"""Main entry point for Sutra AI API server.

Run with:
    python -m main
    or
    uvicorn main:app --reload
"""

import argparse
import logging
import sys
from pathlib import Path

import uvicorn
from sutra_hybrid import SutraAI
from sutra_hybrid.api import create_app

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


def main():
    """Main entry point for the API server."""
    parser = argparse.ArgumentParser(
        description="Sutra AI API Server",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Start server on default port
  python main.py
  
  # Start on custom port
  python main.py --port 8080
  
  # Start with custom storage path
  python main.py --storage ./my_data
  
  # Development mode with auto-reload
  python main.py --reload
  
  # Production mode
  python main.py --workers 4
        """,
    )

    parser.add_argument(
        "--host",
        type=str,
        default="0.0.0.0",
        help="Host to bind to (default: 0.0.0.0)",
    )

    parser.add_argument(
        "--port",
        type=int,
        default=8000,
        help="Port to bind to (default: 8000)",
    )

    parser.add_argument(
        "--storage",
        type=str,
        default="./sutra_data",
        help="Path to storage directory (default: ./sutra_data)",
    )

    parser.add_argument(
        "--reload",
        action="store_true",
        help="Enable auto-reload for development",
    )

    parser.add_argument(
        "--workers",
        type=int,
        default=1,
        help="Number of worker processes (default: 1)",
    )

    parser.add_argument(
        "--log-level",
        type=str,
        default="info",
        choices=["debug", "info", "warning", "error", "critical"],
        help="Logging level (default: info)",
    )

    args = parser.parse_args()

    # Create storage directory if it doesn't exist
    storage_path = Path(args.storage)
    storage_path.mkdir(parents=True, exist_ok=True)

    logger.info(f"Storage path: {storage_path.absolute()}")
    logger.info(f"Starting Sutra AI API server on {args.host}:{args.port}")

    # Initialize AI instance (gRPC)
    storage_server = os.environ.get("SUTRA_STORAGE_SERVER", "storage-server:50051")
    ai = SutraAI(storage_server=storage_server)
    logger.info("SutraAI (gRPC) instance initialized")

    # Create app with pre-configured AI instance
    app = create_app(ai_instance=ai)

    # Run server
    uvicorn.run(
        app,
        host=args.host,
        port=args.port,
        reload=args.reload,
        workers=(
            args.workers if not args.reload else 1
        ),  # Workers don't work with reload
        log_level=args.log_level,
    )


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        logger.info("\nShutting down server...")
        sys.exit(0)
    except Exception as e:
        logger.error(f"Server error: {e}", exc_info=True)
        sys.exit(1)
