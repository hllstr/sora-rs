# sora-on-rust
A Simple, Lightweight and Fast WhatsApp Bot Written on rust.

## How to run
    you need rust nightly toolchain to build this project

    to add your bot number
    $ echo PHONE_NOMBER=628xxxx > .env

    to add your main/sec acc as superuser
    $ echo SUPERUSER=628xxx >> .env

    to run this project as optimized
    $ cargo run --release

    to run this project as debug (faster build process)
    $ cargo run


## Todo
- ai
- translate
- multiple superuser
- better msg logging
- store
  
## Acknowledgemnt
- [whatsapp-rust](https://githuh.com/oxidezap/whatsapp-rust) awesome whatsapp web client api built in rust 
- [sora](https://githuh.com/hllstr/sora) my dead project as inspiration😂
