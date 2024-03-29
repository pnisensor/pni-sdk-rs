![PNI Logo, an ellipse subtracted from the midpoint of a tilted semicircle, and the words PNI Sensor Corporation](https://www.pnicorp.com/wp-content/uploads/PNI-logo-bluewhite-300x161.jpg)

# Rust Compassing SDK
PNI’s compassing and (soon AHRS) modules, including the Prime, TCM, SeaTrax, and Trax, communicate using PNI’s binary protocol. 

## Roadmap
- [ ] feat: AHRS Support
- [ ] Better integration with existing datasheets and documentation
- [ ] More sample code and tests
- [ ] Considering: Flushing serial after every error (may make this opt-in)
- [ ] feat: nicer wrappers for stuff like calibration (to keep track of sample points) and other higher-level abstractions
- [ ] cleanup: Derive on the Get macro, or a more centralized codegen for our SDK
- [ ] feat: Async API
- [ ] feat: support for ASCII devices (will result in several breaking changes!)

## A note about testing
When running `cargo test`, it defaults to running tests in parallel, with the number of jobs being the number of CPUs on your machine.

If tests are performed in parallel, then multiple threads will try to connect to the serialport, leading to a "device busy" failure, and undefined transaction order with the device. 

Please run `cargo test -j1` to limit the number of jobs to 1. Each test should have its own scope and `drop` the serialport (or struct containing it) after it completes its test
