## Raspberry Pi Cross Compiling
If you setup cross compiling on your system, you can run the following...

Otherwise the program will use a mock hardware interface to simulate the pi.

## Setup
Configure your system for cross compilation
``rustup target add armv7-unknown-linux-gnueabihf``

## Build for Pi
``cargo build --target=armv7-unknown-linux-gnueabihf``