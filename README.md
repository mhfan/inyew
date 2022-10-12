
![gh-pages-publish](https://github.com/mhfan/inyew/actions/workflows/publish_gh_pages.yml/badge.svg)

# Study Yew/Rust for Frontend GUI

在[这个 Rust 学习项目](https://github.com/mhfan/inrust)中我收集、整理并分别用 [Rust](https://github.com/mhfan/inrust/blob/master/src/calc24.rs) 和 [C++](https://github.com/mhfan/inrust/blob/master/src/calc24.cpp) 实现了一系列泛化和通用 '24' 点计算问题的简洁算法，还做了一个基于命令行的简单交互程序；

![screen inrust](assets/screen-inrust.jpg)

为了能让一个八九岁、小学四五年级的小朋友 (我可爱的小儿子) 更有兴趣地玩，最好能为基于扑克牌数的经典 24 点计算游戏实现一套足够简洁的图形用户界面；而公开领域能找到的一些 24 点游戏网面在功能和体验上都不太满足我的设想，关键是还没法跑我的算法；

简单研究了 [Slint](https://github.com/slint-ui/slint) 和 [egui](https://github.com/emilk/egui) 之后发现， Rust 世界并没有足够好用能够实现我希望的简洁交互的 GUI 框架，好在 Rust 在 Wasm 世界足够灵活和优秀 (Rust 本来就是从 Mozilla 开始的)，[Html5 + CSS3](https://www.w3schools.com/html/) 应该足以描述任何 GUI 交互形式，于是决定尝试 Rust + Wasm + [Yew](https://yew.rs/)；

所以有了本项目利用 GitHub Pages 技术部署在 [Github.io](https://pages.github.com/) 上，后面加上 [Tauri](https://github.com/tauri-apps/tauri) 也能编译成本地多端应用程序，足以满足各种快速原型的开发需求。

![24 puzzle](assets/screen-24-puzzle.jpg?raw=true)

Based on [Yew Template](https://github.com/Ja-sonYun/yew-template-for-github-io) for [Github.io](https://pages.github.com/), with [tailwind.css](https://tailwindcss.com/) and webpack with your css/scss and [trunk](https://trunkrs.dev) for build.
