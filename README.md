# aamva-rs

A very lenient parser for barcodes as specified by AAMVA for North American
identification cards.

> [!WARNING]
> This library has not been certified in any way and may return incorrect
> information. Additionally, the use of information contained within
> identification cards may be regulated, ensure your uses comply with all local
> laws.

It can easily be used from Rust, or compiled to WebAssembly for websites. An
example of this can be found in the [examples](bindings/aamva-js/examples)
folder. This example can also be found running [here](https://aamva.syfaro.com).

A number of known issues have been worked around to ensure as many IDs as
possible can be decoded as much as possible. Everything has been normalized as
much as possible without lossy conversion.
