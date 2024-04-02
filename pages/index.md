---
title: "BRAILOR's blog"
description: "A blog mainly about Rust and embedded development"
date: "2023-04-22"
hidden: true
---

<img 
    src="https://avatars.githubusercontent.com/u/17928339?v=4" 
    style="border-radius: 100%; width: 100px; margin: 5px 0px 0px 0px; border: 1px solid var(--accent-color);" 
    align="right"
/>
<h1>About me</h1>

Hi I'm Pietro. 

My fields of interest are: embedded systems, cybersecurity, electronics and network programming. 

I work as a <mark>Network Security Engineer</mark> at SynthoHosting. I handle dynamic traffic analysis and build multi-layer filters with eBPF. I'm also the founder of the <mark>Minefence</mark> project: a cloud service for DDoS and bot mitigation for Minecraft servers.

Checkout my **[GitHub](https://github.com/BRA1L0R)** page to get a taste of what I like working on.

### Open source projects:
- [hopper-rs](https://github.com/BRA1L0R/hopper-rs): an L7 reverse proxy for Minecraft. It supports metrics exporting and dynamic reloading.
- [netherite-rs](https://github.com/BRA1L0R/netherite-rs): Rust library for the Minecraft protocol. It has all the basic building blocks for implementing your own packets through procedural macros.
- [morse-gadget](https://github.com/BRA1L0R/morse-gadget): A very elaborate embedded Valentine's day gift. A blog post about it is coming soon
- [deezer-bot](https://github.com/Stockpesce/deezer-bot): Open source alternative to many music bots on Telegram. 
- [hyperlight](https://github.com/hyperlightjs/hyperlight) (discontinued): A JavaScript framework for building server side rendered applications with Hyperapp.


<br>

{{#each pages}}
{{#unless metadata.hidden}}
<h2 style="margin-bottom: 0px;"><a href="{{figurative_path}}">{{metadata.title}}</a></h2>
<span style="font-size: 15px; color: grey;">{{metadata.date}}</span>

{{metadata.description}}

[Read more]({{figurative_path}})

---

{{/unless}}
{{/each}}