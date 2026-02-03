from setuptools import setup, find_packages

setup(
    name="sutra-engine-client",
    version="1.0.0",
    description="Python client for the Sutra Engine standalone reasoning engine.",
    author="Sutra Works",
    packages=find_packages(),
    install_requires=[
        "msgpack>=1.0.0",
    ],
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
)
