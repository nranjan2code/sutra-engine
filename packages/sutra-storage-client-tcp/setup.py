from setuptools import setup, find_packages

setup(
    name="sutra-storage-client",
    version="2.0.0",
    description="Storage client using custom binary protocol (replaces gRPC)",
    packages=find_packages(),
    install_requires=[
        "msgpack>=1.0.0",  # Fast binary serialization
    ],
    python_requires=">=3.8",
)
