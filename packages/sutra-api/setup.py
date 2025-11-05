"""Setup configuration for sutra-api package."""

from setuptools import find_packages, setup

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="sutra-api",
    version="1.0.0",
    author="Sutra AI Team",
    description="REST API service for Sutra AI graph reasoning system",
    long_description=long_description,
    long_description_content_type="text/markdown",
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=[
        "sutra-core==1.0.0",
        "sutra-hybrid==1.0.0",
        "fastapi==0.115.0",
        "uvicorn[standard]==0.30.6",
        "pydantic==2.9.2",
        "python-multipart==0.0.12",
    ],
    extras_require={
        "dev": [
            "pytest==8.3.3",
            "pytest-asyncio==0.24.0",
            "httpx==0.27.2",
            "black==24.8.0",
            "isort==5.13.2",
            "flake8==7.1.1",
            "mypy==1.11.2",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
)
