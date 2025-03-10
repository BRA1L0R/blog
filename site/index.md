---
title: "BRAILOR's blog"
description: "A blog mainly about Rust and embedded development"
date: "2023-04-22"
hidden: true
---

<img
    src="/static/heathen.jpg"
    style="display: inline-block; box-sizing: border-box; border-radius: 100%; width: 100px; margin: 0px -5px 0px 0px; border: 1.5px solid var(--accent-color);"
    align="right"
/>
<h1 class="fancy-title">About me</h1>

Hi I'm Pietro (18M).

My fields of interest are: embedded systems, cybersecurity, electronics and network programming.

I work mainly as freelancer but I maintain open-source in my spare time.

Here are some useful contact links: [**LinkedIn**](https://www.linkedin.com/in/pietro-tamilia-3a9168238/), [**Telegram**](https://t.me/stack_smash). Checkout my **[GitHub](https://github.com/BRA1L0R)** page to get a taste of what I like working on.

<br>

### My open source projects:
<!-- <details open> -->
<!-- <summary>A list of projects I'm working/I've worked on</summary> -->

- **Linux / Networking**:
  - [xdp-loader](https://github.com/BRA1L0R/xdp-loader): WIP xdp loader with map pinning and configurable program array dispatching capabilities.
  - [hopper-rs](https://github.com/BRA1L0R/hopper-rs): an L7 reverse proxy for Minecraft. It supports metrics exporting and dynamic reloading.
  - [netherite-rs](https://github.com/BRA1L0R/netherite-rs): Rust library for the Minecraft protocol. It has all the basic building blocks for implementing your own packets through procedural macros.
- **Embedded**:
  - [morse-gadget](https://github.com/BRA1L0R/morse-gadget): A very elaborate electronics-related Valentine's day gift. A blog post about it is coming soon
  - [alvik-idf](https://github.com/BRA1L0R/alvik-idf-rs): Library for interfacing with the Alvik hardware on esp32 with IDF on Rust.
  - [ucpack](https://github.com/BRA1L0R/ucpack): Body protocol of the Arduino Alvik
- **Miscellaneous**:
  - [deezer-bot](https://github.com/Stockpesce/deezer-bot): Open source alternative to many music bots on Telegram.
  - [hyperlight](https://github.com/hyperlightjs/hyperlight) (discontinued): A JavaScript framework for building server side rendered applications with Hyperapp.

<!-- </details> -->

<br>
<br>

<h1 class="fancy-title">Blog Posts</h1>

{{#each pages}}
{{#unless metadata.hidden}}
<h2 style="margin-bottom: 0px;"><a href="{{figurative_path}}">{{metadata.title}}</a></h2>
<span style="font-size: 15px; color: grey;">{{metadata.date}}</span>

{{metadata.description}}

[Read more]({{figurative_path}})

---

{{/unless}}
{{/each}}
