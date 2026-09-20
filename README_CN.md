# CometixCode

[English](README.md) | 简体中文

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)
![License](https://img.shields.io/badge/License-AGPL--3.0-blue)

一个终端 AI 编码助手，用 Rust 对 Anthropic Claude Code 做 1:1 复刻。

> **非官方项目，与 Anthropic 无隶属关系。**
> 本项目是通过研读已发布的 Claude Code CLI 独立实现的。
> "Claude" 与 "Claude Code" 是 Anthropic, PBC 的商标。
> 本项目未获得 Anthropic 的背书或支持。

## 这是什么

Claude Code 的终端界面是 TypeScript + React + Ink。CometixCode 在 Rust 上用
iocraft 复刻它 —— 那是一个 React 风格的保留模式 TUI 框架，是 Ink 的结构对应物：
同样有组件、hooks 和声明式元素。移植是逐文件对照原版进行的，而不是另起炉灶重新
设计架构：一个 TypeScript 组件对应一个 Rust 组件，一个 hook 对应一个 hook，刻意
的偏离都就地记录在源码注释里。

项目构建于 [CometixTUI]，那是 iocraft 的一个 fork，携带了本移植所需的原语
（行级差分、事件传播、SIGCONT 自愈、输入法光标、括号粘贴、网格布局）。

项目仍在进行中。交互主循环、工具执行、权限、MCP 与斜杠命令的大部分已经实现，
其余部分尚不完整。

## 构建

### 前置依赖

- **Rust 1.87+**（edition 2024）
- **ICU4C** —— `rust_icu_*` 系列 crate 通过 `pkg-config` 绑定它

在 macOS 上 ICU4C 是 keg-only 的，需要把它的 `pkgconfig` 目录加入搜索路径：

```sh
brew install icu4c
export PKG_CONFIG_PATH="$(brew --prefix icu4c)/lib/pkgconfig"
```

在 Debian/Ubuntu 上：

```sh
sudo apt install libicu-dev pkg-config
```

### 构建与运行

```sh
cargo build --release
cargo run --release
```

可执行文件名为 `cometix`。

## ripgrep

文件搜索会调用外部的 `rg`。Claude Code 把打包好的 ripgrep 放在自己的 npm 包内
并执行那个路径；CometixCode 保持同样的形态 —— 该二进制是外挂资产，**不会**被编译
进可执行文件。

查找顺序：

1. 可执行文件同级的 `vendor/ripgrep/<arch>-<os>/rg`（发行包）
2. 源码树中的 `vendor/ripgrep/<arch>-<os>/rg`（开发环境）
3. `PATH` 中宿主自带的 `rg`

所以你既可以在 `vendor/ripgrep/` 下放一份固定版本的 ripgrep，也可以直接装一个
`rg` 了事。`/doctor` 会报告当前用的是哪一个。设置 `USE_BUILTIN_RIPGREP=0` 可强制
使用宿主的二进制。

## 测试

```sh
just test          # cargo-nextest 全量套件 —— 唯一的门禁
just t <pattern>   # 按子串过滤
just check         # 仅做类型/借用检查，不链接
```

`cargo test` **不是**本 crate 的有效门禁。它把整个套件作为线程跑在同一个进程里，
而这份代码库存在进程级状态（环境变量、`OnceLock` 缓存），因此会在测试之间泄漏。
`just test` 走 `cargo-nextest`，为每个测试分配独立进程，并钉住测试本会从宿主继承
的那部分状态。justfile 的头部注释记录了支撑这一判断的实测数据。

## 社区

<a href="https://qm.qq.com/q/nmbS5eUUP8" target="_blank"><img src="https://img.shields.io/badge/QQ%20群-1045122926-EB1923?logo=tencentqq&logoColor=white" alt="QQ Group" /></a>
<a href="https://t.me/CometixSpace" target="_blank"><img alt="telegram" src="https://img.shields.io/badge/chat-telegram-blueviolet?style=flat&logo=Telegram"></a>
[![LINUX DO](https://img.shields.io/badge/LINUX%20DO-Community-blue)](https://linux.do/t/topic/2927016)

## 致谢

- **[Anthropic]** —— 感谢 Claude Code，本项目复刻的对象。这里的每一处行为都源自
  对它的研读，设计上的功劳属于他们。
- **[iocraft]**（作者 ccbrown）—— 本项目所基于的 React 风格保留模式 TUI 框架，
  [CometixTUI] 是它的 fork。
- **[ripgrep]**（作者 BurntSushi）—— `Grep`、`Glob` 以及各条文件发现路径最终调用
  的搜索工具。
- **[nucleo]**（来自 Helix 编辑器项目）—— 文件、命令与 agent 补全背后的模糊匹配器。

## 许可

[AGPL-3.0-only](LICENSE)。

请注意其网络条款：如果你把修改后的版本作为网络服务运行，其使用者有权获得修改后的
源码。

[CometixTUI]: https://github.com/Haleclipse/CometixTUI
[Anthropic]: https://www.anthropic.com
[iocraft]: https://github.com/ccbrown/iocraft
[ripgrep]: https://github.com/BurntSushi/ripgrep
[nucleo]: https://github.com/helix-editor/nucleo
