---
title: "Homepage"
description: ""
date: "2023-04-22"
hidden: true
---

{{#each pages}}
{{#unless metadata.hidden}}
<h2 style="margin-bottom: 0px;"><a href="{{figurative_path}}">{{metadata.title}}</a></h2>
<span style="font-size: 15px; color: grey;">{{metadata.date}}</span>

{{metadata.description}}

[Read more]({{figurative_path}})

---
{{/unless}}
{{/each}}