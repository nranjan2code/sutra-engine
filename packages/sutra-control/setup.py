"""Setup for sutra-control package."""

from pathlib import Path

from setuptools import find_packages, setup

# Read README
readme_file = Path(__file__).parent / "README.md"
long_description = readme_file.read_text() if readme_file.exists() else ""

setup(
    name="sutra-control",
    version="0.1.0",
    description="Web-based control center for Sutra AI system lifecycle management",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Sutra AI Team",
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=[
        "fastapi>=0.104.0",
        "uvicorn[standard]>=0.24.0",
        "psutil>=5.9.0",
        "websockets>=12.0",
        "pydantic>=2.0.0",
    ],
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "black>=23.0.0",
            "isort>=5.12.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "sutra-control=sutra_control.main:app",
        ],
    },
    include_package_data=True,
    package_data={
        "sutra_control": [
            "../static/css/*.css",
            "../static/js/*.js",
            "../templates/*.html",
        ],
    },
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
)
