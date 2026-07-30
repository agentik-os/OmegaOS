# OmegaOS

一个终端控制平面，用于并行运行一支 AI 编码 agent 编队，编队里的每个 agent 都遵守同一套类型化规则手册。

[English](README.md) | [Français](README.fr.md) | [Русский](README.ru.md) | 中文

> [英文版 README](README.md) 是权威且最新的版本；本翻译可能略有滞后。

[![CI](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml/badge.svg)](https://github.com/agentik-os/OmegaOS/actions/workflows/ci.yml) ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg) ![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange.svg)

OmegaOS 不是一个供你 import 的库。你把它装在一台 Linux 机器上，得到的是 `omega` 命令、一个用来盯着会话并随手 kill 掉它们的 TUI，以及一层把活儿派给 agent 的编排逻辑。还附带一个 Telegram 桥接，方便你用手机来驱动它。

默认的 agent 运行时是 OpenAI Codex。Claude Code、Gemini、Pi、Hermes 和 GLM 仍可显式选择。每个 agent 都会收到由同一 doctrine 编译出的紧凑、类型化、按角色裁剪的上下文。

当前版本见 [CHANGELOG.md](CHANGELOG.md)（在已安装的机器上运行 `omega -V`）。我每天都在用它，请预期会有些粗糙的地方。

## doctrine

有一个类型化的注册表，包含 7 条 Law 和 47 条具名的操作 Rule。`omega rules list` 会打印当前集合。

**Law 不可违背。**它们约束每一个 agent，并且凌驾于每一条 rule、每一个 task 之上。一共七条：

- **L0 — Ship the truth（交付真相）。**一处改动，只有在一次干净的重新构建能复现它、并且它已被 push 之后，才算完成。差一点的都只是草稿。
- **L1 — Runtime is the only truth（运行时是唯一的真相）。**代码和注释陈述的是意图，只有真正跑起来才会显出现实。两者不一致时，运行时说了算。
- **L2 — Researcher, not sycophant（做研究者，别做马屁精）。**遇到有缺陷的前提，先用推理去质疑它，再动手。不要装出来的自信。"这应该能行"而没有证据，就是撒谎。
- **L3 — Decide and proceed（决断并推进）。**被派发出去的 agent 是自主的。它绝不停下来问"我该继续吗？"它自己决断、自己执行、事后再汇报。
- **L4 — Done means 100%, verified（完成意味着 100%，且经过验证）。**92% 不叫完成。把任务逐条列出来，逐条做完，逐条对着运行时验证。
- **L5 — Quality over speed（质量高于速度）。**真协议不存在精简版、轻量版或快速版。403 或 401 是中止，不是通过。
- **L6 - Finish the mission（完成整项任务）。**列出、执行、验证并报告每一个请求的交付物。计划或部分阶段不是合法的停止点。

**Rule 是操作层面的。**它们是具名的（R-SCOPE、R-VERIFY、R-CITE……），归入 Universal、QualityGate、Orchestration、Reporting、Safety 几类。每条 Rule 都按它所约束的角色来划定范围：Master、Oracle、Worker。一个 worker 不会被它根本无从下手的编排规则压上身，一个 oracle 也不会背上 worker 那套文件加锁的纪律。同一个注册表，切出不同的片。

### 漏斗

机制由 `rules::compile_rule_context_for_provider` 实现：组合紧凑的 Law 核心、角色契约、任务相关 Rule 和 provider 机制。超过 24 KB 的 OmegaOS 上下文会被拒绝，而不是静默截断。

一个钻到树里三层深的 worker，带着的七条 Law 和顶端的 Master 一模一样。谁也没法悄悄生一个把 L5 偷偷丢掉、好跑得更快的子节点——因为子节点的 prompt 是由同一个函数、从同一个注册表里组装出来的。

正因为 doctrine 只是文本，所以无论后端是 Claude、GPT、Gemini，还是你以后加进来的别的东西，它的运作方式都一样。

看完整内容：

```
omega rules list
```

![omega rules list —— OmegaOS 输出的 Law 与 Rule](assets/omega-rules.svg)

## 架构

四层，自上而下。

**第 1 层 —— 人机界面。**TUI、CLI（40+ 命令）和 Telegram 桥接,在底下驱动的都是同一层。

**第 2 层 —— AISB Master。**一个常驻的 agent，保持运行，挂了会自动重启，并用 `--continue` 续上它自己的对话。它内置 14 个以《黑客帝国》角色命名的 agent 模板（Oracle、Morpheus、Seraph、Keymaker、Smith、Niobe、Architect、Merovingian、Neo、Zion、Link、Construct、Pythia、Council）。Master 是个派发器。它只做分类，把活儿路由给各个 oracle。

**第 3 层 —— Oracle。**每个项目一个。它给请求分类、做规划、派发 worker，并在最后跑质量门禁。一个 oracle 负责编排，它自己不动项目代码。

**第 4 层 —— Worker。**短命的。它们并行运行，每一个都被一份文件锁声明圈定到它自己的那些文件上：一个文件一个写入者，用咨询式文件锁（fs2）强制执行。这把锁是实打实的，不是君子约定。一个 worker 通过写一个 `done.json`、把 status 标成 `done_clean`、`pending` 或 `failed` 来发出完成信号；没有这个 status，它就不算完成。

## 一个任务是怎么跑起来的

一个请求可以从三个入口进来：TUI、`omega` CLI，或者 Telegram 桥接。无论从哪里出发，它最终都会落到 AISB Master 手里。Master 读取它、分类它，再把它路由到负责相关项目的那个 oracle。它不碰任何文件。它的全部职责就是决定活儿该往哪儿走。

接下来由 oracle 接手。它名下只管一个项目；它来规划任务、把任务拆成若干子任务，然后给每个子任务派发一个 worker。当各个 worker 汇报回来，它跑一道质量门禁，再把结果向上逐级汇报。它唯一不会做的事，就是改动该项目的代码。打分的 agent 和写代码的 agent 不是同一个，所以这个分数不是自评。正因为打分者没写过这段代码，它给的分才是独立的。

worker 都是短生命周期的，并行运行。一个 worker 在写某个文件之前，会先用一把建议锁（通过 `fs2`）把那个文件占住，所以两个 worker 在物理上根本没法同时写同一个文件。这样就保证了每个文件只有一个写入者——靠的是锁，而不是靠约定。worker 做完自己那份活，对照真实运行时核对结果，然后写出一份 `done.json`，状态是 `done_clean`、`pending` 或 `failed` 之一。oracle 读这份文件，确认，然后关闭会话。只要状态不是 `done_clean`，任务就不算完成。

worker 并不非得一个一个地啃自己的子任务。它可以在进程内跑一个 Workflow：派生一批并行的子 agent，核对它们的输出，再把它们汇成一个答案。代码评审就是这么干的，调研、审计和设计工作也一样。这通常比为每个子任务都派发一个全新 worker 更省，而且产出更好。

验证是刻意做成对抗式的：一个 worker 报了「done」并不会就此结束核查；它的说法仍然必须被验证。每条说法都交给若干独立的 agent，只有在多数（三取二）认可时它才能存活下来。每一条结论在被接纳之前，都要和其他 agent 互相比对核验。Quality Arsenal 那套审计，正是接在这里——接在门禁这一环。

这一切都依赖前面讲的那个 doctrine 漏斗：每个 agent，在每一层级，一被派发,就会立刻被注入按角色裁剪过的那套 Laws 和 Rules。往下三层的一个 worker，拿到的 L0–L5 规则和 Master 拿到的一模一样。

这一节 README 本身就是个例子。它是一个 Workflow 产出的。一个 agent 写了初稿，几个独立的审阅者逐句过了一遍、专门揪那些一看就是 AI 写出来的腔调，另一个 agent 对着他们标出来的地方做了修订，再由母语者完成翻译。所以这段文字没有哪一部分是出自某一次没人复核的单遍生成。

## 技术栈

这是一个 Rust workspace，含三个 crate：

- `omega-core` —— 编排、规则注册表、doctor、timeline、cleanup、patrol、文件范围加锁。
- `omega-cli` —— `omega` 二进制，构建在 `clap` 之上。
- `omega-tui` —— 会话管理器，构建在 `ratatui` 之上。

再往下，它跑在 [rmux](https://github.com/agentik-os/rmux) 上——一个 Rust 写的终端复用器：一个守护进程、一套类型化 SDK，外加 PTY 处理。rmux 是一个类型化的 Rust 库，所以 OmegaOS 直接调用它，而不是去 shell 出 tmux 再解析文本。整个项目任何地方都没有对 tmux 的依赖。

Bun 和 TypeScript 负责 PDF 报告的渲染，经由 Next.js 和 Playwright。Bash 只在一个地方露面：安装的引导脚本。

## 安装

你需要一台 Linux 机器和一套 Rust 工具链。安装器会从源码构建 `rmux` 和 `omega`，所以第一次跑起来不会是瞬间完成的。

```
git clone https://github.com/agentik-os/OmegaOS
cd OmegaOS
./install.sh
```

跑完之后，运行 doctor。

## 用法

`omega doctor` 是第一个该跑的命令，也是一旦觉得哪里不对劲就该跑的命令。它会把整个技术栈检查一遍：

```
OmegaOS doctor

  [+] binary           omega 0.1.5
  [+] rmux daemon      connected, 6 live session(s)
  [+] rmux socket      /tmp/rmux-1000/default
  [+] doctrine         7 Laws + 47 Rules
  [+] agent CLI        codex available
  [+] state dir        /home/vibe/.omega/state
  [+] telegram service omega-tg-bot active
  [+] hooks            track + verify present, registered in settings.json
  [+] secrets dir      /home/vibe/.omega present
  [+] memory           249088MB available
  [+] codex auth       Codex login valid
  [+] telegram poller  1 poller
```

`[!]` 那几行是警告，不是错误——每一行都自带修复命令，`omega doctor --fix` 会自动修好机械性的问题。

命令面，挑你实际真会用到的列一下：

```
omega menu          启动 TUI 会话管理器
omega doctor        对整个技术栈做一次性健康检查
omega rules list    列出 Law 与 Rule
omega dispatch      把一项任务派发给某个 oracle
omega orchestrate   端到端跑完一整项任务（分类、规划、派发、监控、门禁）
omega spawn-worker  在当前 oracle 之下生出一个 worker
omega team          在分屏窗格里生出一队 agent
omega done          发出任务完成信号（由 worker 调用）
omega timeline      回放某个 oracle 从派发到完成的历史
omega resurrect     从持久化状态里把一个崩掉的 oracle 重新生出来
omega cleanup       对游离会话和过期状态做核弹级清理
omega backup        把无法复现的 ~/.omega 状态备份成单个 tgz
omega telegram      管理 Telegram 桥接
omega pdf           生成一份 PDF 报告
```

`omega menu` 打开 TUI。rmux 守护进程拥有每一个 PTY，每个会话都带一个角色：Master、Oracle、Worker、Home（你自己的交互式 shell）或 System（像 Telegram 桥接这类守护进程）。TUI 把它们列出来，附带实时进度，让你能 kill、加锁、重命名。里头内置了 kill-all、一个状态变陈旧时用的核弹级清理，还有 doctor。

日常的循环很小。`omega orchestrate` 端到端跑完一整项任务。`omega timeline` 一次一次派发地回放某个 oracle 干过的事。而当一个 oracle 崩了，`omega resurrect` 会把它从持久化状态里救回来。

## 质量军火库

在 `skills/audits/` 下有大约两打法证级审计——`codeaudit`、`secaudit`、`perfaudit`、`a11yaudit`、`uiuxaudit`、`flowaudit`、`seoaudit`、`apiaudit` 等等。每一个都跑一套 Gestalt-Popper 协议：先是一道清晰度门禁，接着是主动证伪——审计会去想方设法把这东西弄坏，而不是去印证它——然后对那个最要紧的单点施以 10 倍的审视，而不是把注意力均摊出去。一个 oracle 跑完一项任务时，会自动挑出与刚刚改动相匹配的那些审计，省得你去记该跑哪些。

## 局限

这些事，我宁愿你进来之前就知道。

- **Linux 优先。**在一台无头 VPS 上开发。没有 Windows。macOS 未经测试，但大体上应该能用，毕竟它就是 Rust 加 rmux。
- TUI 假定终端支持 256 色。在 16 色终端上它会很丑。
- 默认的 agent 运行时是 OpenAI Codex，因此 `codex` CLI 必须已登录。Claude Code、Gemini、Pi、Hermes 和 GLM 仍是显式可选项。
- **单机。**rmux 守护进程是本地的。没有跨主机的编排。
- 这是 0.1.x。我每天都用，但你会撞上一些我还没撞过的粗糙地方。

## 致谢

OmegaOS 建立在许多其他人的工作之上：

最大的一笔债是 [rmux](https://github.com/agentik-os/rmux)，这里一切都跑在它上面的那个 Rust 终端复用器。

Rust 技术栈的其余部分：

- [ratatui](https://github.com/ratatui/ratatui) 和 [crossterm](https://github.com/crossterm-rs/crossterm) —— TUI。
- [tokio](https://github.com/tokio-rs/tokio) —— 异步运行时。
- [clap](https://github.com/clap-rs/clap) 和 `clap_complete` —— CLI 与 shell 补全。
- [serde](https://github.com/serde-rs/serde)，配合 `serde_json`、`serde_yaml` 和 `toml` —— 配置与状态。
- [anyhow](https://github.com/dtolnay/anyhow) 和 [thiserror](https://github.com/dtolnay/thiserror) —— 错误处理。
- `chrono`（时间戳）、`dirs`（路径）、`fs2`（范围声明背后的那些咨询式文件锁）、`regex`、`tempfile`、`tracing` 配合 `tracing-subscriber`（日志），以及 `reqwest`（Telegram 与 PDF 的 HTTP）。

Anthropic 出品的 [Claude Code](https://www.anthropic.com) 是 agent 运行时。

## 许可证

双许可，[MIT](LICENSE-MIT) 与 [Apache-2.0](LICENSE-APACHE) 任选其一，悉听尊便。标准的 Rust 惯例。挑你顺眼的那个就行。
