# Micro:bit V2 Examples

Exercises taken from [Discovery Book: Embedded Rust](https://docs.rust-embedded.org/discovery/microbit/index.html)

All exercises were re-worked to remove any bits of unsafe code & add better error handling
Also expanded functionality so it is a bit more advanced :)

## Examples
Original article has multiple examples, but those are more about learning one's way around Embedded concepts, so I'm trying to incorporate a bit of fun in there to ensure I fully understand how Embedded world works
1. `i2c`
2. `led_compass`
3. `led_roulette`
4. `punch_o_meter`
5. `scrolling_led`


## Use
1. Select example
2. Then from example folder execute
```
# To build & flash binaries
cargo embed --target thumbv7em-none-eabihf --release

# To find out how big is the binary
cargo size --target thumbv7em-none-eabihf --release -- -A
```
