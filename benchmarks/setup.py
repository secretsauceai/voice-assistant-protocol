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

        'cycler',                     # Needed by matplotlib
        'kiwisolver',                 # Needed by matplotlib
        'numpy',                      # Needed by matplotlib
        'packaging',                  # Needed by matplotlib
        'pillow',                     # Needed by matplotlib
        'pyparsing',                  # Needed by matplotlib
        'python-dateutil',            # Needed by matplotlib
        

        # Protocols
        'aiocoap',                    # For CoAP protocol
        'paho-mqtt',                  # For MQTT protocol
        'mqttsn',                     # For MQTT-SN protocol
        
        'msgpack',                    # For MessagePack messages
        'flatbuffers',                # For FlexBuffers messages
        'mapbuffer',                  # For MapBuf messages

        'brotli',                     # Dependency of mapbuffer
        'crc32c',                     # Dependency of mapbuffer
        'deflate',                    # Dependency of mapbuffer
        'tqdm',                       # Dependency of mapbuffer
        'zstandard',                  # Dependency of mapbuffer

        'rhasspy-hermes',             # For Hermes protocol

        ## Hivemind protocol
        'hivemind_bus_client',
        'mycroft_messagebus_client',  # For Hivemind and Mycroft communication

        'inflection',                 # Dependency of hivemind
        'json-database',              # Dependency of hivemind
        'kthread',                    # Dependency of hivemind
        'pexpect',                    # Dependency of hivemind
        'requests',                   # Dependency of hivemind

        'filelock',                   # Dependency of json-database
        'memory_tempfile',            # Dependency of json-database
        
        
    ])