Maiden
======
[![Build Status](https://travis-ci.org/palfrey/maiden.svg?branch=master)](https://travis-ci.org/palfrey/maiden)

Maiden is a [Rockstar](https://github.com/dylanbeattie/rockstar) interpreter written in Rust. 

Why the name?
-------------
Well given that Rockstar is about hard rock from the 1980s, [Iron Maiden](https://en.wikipedia.org/wiki/Iron_Maiden) seemed like the obvious name given the existing use of [iron](http://ironframework.io/) as a name for Rust programs (despite it actually being named after [a fungus](https://en.wikipedia.org/wiki/Rust_%28fungus%29))

Status
------
Experimental. Rockstar is still very much in active flux, and Maiden doesn't even implement the [entire current spec](https://github.com/dylanbeattie/rockstar) as of writing this, but I'm seeing what I can do.

Usage
-----
`cargo run --quiet <your rockstar program>` works pretty well

Web version
-----------
There's a deployed edition at https://palfrey.github.io/maiden/. To work with it
1. [Install Rust nightly](https://github.com/rust-lang-nursery/rustup.rs/#working-with-nightly-rust) (to work around stdweb needing procedural macros. See also https://github.com/rust-lang/rust/issues/38356)
2. [Install cargo web](https://github.com/koute/cargo-web#installation)
3. Run `cargo +nightly web start --auto-reload`
4. Goto http://localhost:8000/