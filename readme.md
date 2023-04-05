# mori-summer-mqtt

This project is just a simple rust app that connects any machine to a mqtt broker.

The purpose of this is to post a `hello/<hostname>` message to the broker on connect, and set a last will and testament to `death/<hostname>`.

## Building

Command I used to build this:

`cross build --target=aarch64-linux-android --release`
