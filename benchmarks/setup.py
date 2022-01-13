from setuptools import setup

setup(name="protocol-benchmark",
    version="0.1",
    description="Benchmark of protocols",
    url="...",
    author="Sergio Tortosa",
    license="MIT",
    install_requires=[
        # Benchwork
        'matplotlib',
        'memory_profiler',
        'psutil',
        

        # Protocols
        'aiocoap',                    # For CoAP protocol
        'paho-mqtt',                  # For MQTT protocol
        'mqttsn',                     # For MQTT-SN protocol
        
        'msgpack',                    # For MessagePack messages
        'flatbuffers',                # For FlexBuffers messages
        'mapbuffer',                  # For MapBuf messages
        'brotli',                     # Dependency of mapbuffer
        'deflate',                    # Dependency of mapbuffer
        'zstandard',                  # Dependency of mapbuffer
        'tqdm',                       # Dependency of mapbuffer

        'rhasspy-hermes',             # For Hermes protocol

        ## Hivemind protocol
        'hivemind_bus_client',
        'mycroft_messagebus_client',  # For Hivemind and Mycroft communication

        'kthread',                    # Dependency of hivemind
        'inflection',                 # Dependency of hivemind
        'json-database',              # Dependency of hivemind
        
    ])