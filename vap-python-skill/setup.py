from setuptools import setup

setup(name="vap-python-skill",
    version="0.1",
    description="Example of a Python skill",
    url="...",
    author="Sergio Tortosa",
    license="MIT",
    install_requires=[        
        # Protocols
        'aiocoap',
        'msgpack',        
    ])