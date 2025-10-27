"""
Setup configuration for sutra-ml-base package
"""

from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

with open("requirements.txt", "r", encoding="utf-8") as fh:
    requirements = [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="sutra-ml-base",
    version="2.0.0",
    author="Sutra AI Team",
    author_email="team@sutra.ai",
    description="Foundation library for Sutra AI ML services",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/nranjan2code/sutra-memory",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    python_requires=">=3.11",
    install_requires=requirements,
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.0.0",
            "black>=23.0.0",
            "isort>=5.12.0",
            "flake8>=6.0.0",
            "mypy>=1.5.0",
        ],
        "optimized": [
            "accelerate>=0.25.0",
            "sentencepiece>=0.1.99",
            "protobuf>=3.20.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "sutra-ml-validate=sutra_ml_base.cli:validate_model",
            "sutra-ml-benchmark=sutra_ml_base.cli:benchmark_service",
        ],
    },
    include_package_data=True,
    zip_safe=False,
)