# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [1.5.2] - 2025-07-24

### 🐛 Fixed

- 更新后未保存被消耗的版本状态。
- `cargo release` 过程中错误地替换了 ReadMe 中的版本信息。

## [1.5.1] - 2025-07-24

### ⚙️ Changed

- 优化 ChangeLog 流程。实现全流程自动化。应用 CHANGELOG 标准。

### ⛔ Removed

- 禁用目标低于 1.5.0 版本的更新。

## [1.5.0] - 2025-07-24

### 🚀 Added

- 自我更新功能，允许用户通过 `fp update` 进行更新或自动更新。

### ⚙️ Changed

- 在 fp build 过程中，在可用阶段输出所使用的 CL。若无法获取则输出警告。

## [1.4.6] - 2025-07-22

### ⚙️ Changed

- 更新打包流程，预期适应 self-update 标准。

## [1.4.5] - 2025-07-21

### 🚀 Added

- 优化 `fp watch` `fp build` 的输出，使用户能直接访问原链接。

### ⚙️ Changed

- 优化针对 jenkins url 记忆的空值处理。
- 优化 `fp watch` 在无法找到用户自己的 run task 时，则进入二次输入流程。
- 优化登录错误提示。

## [1.4.4] - 2025-06-10

### 🚀 Added

- 添加对 xml 中的富文本支持。当前仅支持了 `<span>` 标签。
- 允许 `fp build` 与 `fp watch` 通过 `-[PARAM]` 的方式传入其他 extract 参数。
- 统一错误处理。

### ⚙️ Changed

- 优化 `fp build` 所使用的 CL 的记忆，延长生命周期以保持可用性。

### 🐛 Fixed

- `fp extract` 时错误的文件生成路径。该问题曾导致在 fp 的工作目录生成了一些空文件夹。

## [1.4.3] - 2025-05-09

### 🚀 Added

- 在需要 Login 的命令中，添加首次登录检查，以直接跳转至登录流程。

## [1.4.2] - 2025-05-09

### ⚙️ Changed

- 优化代码结构。

### 🐛 Fixed

- 未正确查询 Jenkins 最新打包所使用的 CL。该问题导致 CL 不再作为记忆参数被提供。

## [1.4.1] - 2025-05-06

### 🚀 Added

- 在构建错误并输出日志前，展示 url 供直接跳转。

### 🐛 Fixed

- 修复当多个重复状态的 Run Task 时，无法正确获取最新包的问题。

## [1.4.0] - 2025-05-06

### 🚀 Added

- Build 功能。一键发起 Jenkins Build，包含自动查询默认参数、填充推荐参数、智能记忆以及操作链。

### ⚙️ Changed

- 在 fp watch 时主动询问 Job Name。
- 支持自定义 Job Name.

## [1.3.11] - 2025-04-23

### ⛔ Removed

- Cookie 登录的支持。添加了密码登录。

## [1.3.10] - 2025-04-22

### 🚀 Added

- watch 功能。它会在 Jenkins 上有新包时，自动下载并解压。

### ⛔ Removed

- Login 命令时请求 job_name 的输入。现在流程上不再必要。

### 🐛 Fixed

- 错误的 build number of last run checking。该错误导致查询用户上次解压包时，若该包不是最新包，则错误推断为不存在。
- 在输入 job name 时，未根据上次输入对 Jenkins 进行排序。

## [1.3.9] - 2025-04-18

### 🚀 Added

- 添加调试模式。

### ⚙️ Changed

- 提前存储成功登陆后的 Jenkins 信息。

## [1.3.8] - 2025-04-18

### 🚀 Added

- 添加查询用户最新包时的等待提示文本。
- 向查询用户最新包添加额外的信息：构建中、构建失败。

### ⚙️ Changed

- 优化了当 Jenkins 登录状态疑似过期时的提示。
- Jenkins 相关功能结构调整。
- 优化输出颜色，使其更具表达性。
- 调整操作历史存储，取消 branch 字段，合并功能为 interest_job_name 字段。
- 持续优化模块化输入函数。

## [1.3.7] - 2025-04-10

### ⚙️ Changed

- `main.rs` 中的代码。使用模块化的函数代替过程。

### 🐛 Fixed

- 当 RunInfo 结果为 null 时引发的解析错误。该错误曾导致当有进行中的 Run Task 时，无法获得用户当前的最新已完成任务。

## [1.3.6] - 2025-04-08

### 🚀 Added

- 与 Jenkins 进行数据沟通的能力。使用 `fp login` 选择一种方式进行登录，随后使用 `fp extract` 时将快速得到专属于你的包。
    - 推荐 Cookie 登录而非 ApiToken。ApiToken 非常的慢！

## [1.3.5] - 2025-03-31

### 🚀 Added

- 全新的运行存储管理，使得对不同版本的数据具有更好的兼容性与开发可读性。

### 🐛 Fixed

- 一些错误的单元测试配置。

## [1.3.4] - 2025-02-12

### 🚀 Added

- 重置 `fp run -S` 的方式。当用户未输入 `-S` 参数时，将采用 localhost 作为服务器。

### 🐛 Fixed

- 模拟 index 的偏移。该修复可能导致原有的账号信息反偏移。

## [1.3.3] - 2024-12-11

### ⚙️ Changed

- 使用环境变量配置敏感数据。
- 优化 pretty log 结构，使其更具表达性。
    - 添加了 VfpPrettyLogger 结构，使用其构造函数替换原本的 `pretty_log_operation_start` 函数。
    - 使 `pretty_log_operation_status` 被调用前必须调用 `pretty_log_operation_start` 成为一种可靠保证。

## [1.3.2] - 2024-12-10

### 🚀 Added

- 打通 Github CI 流程。

<!-- next-url -->

[Unreleased]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.5.2...HEAD

[1.5.2]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.5.1...v1.5.2

[1.5.1]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.5.0...v1.5.1

[1.3.2]: https://github.com/LviatYi/vertical-fire-platform/releases/tag/v1.3.2

[1.3.3]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.2...v1.3.3

[1.3.4]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.3...v1.3.4

[1.3.5]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.4...v1.3.5

[1.3.6]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.5...v1.3.6

[1.3.7]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.6...v1.3.7

[1.3.8]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.7...v1.3.8

[1.3.9]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.8...v1.3.9

[1.3.10]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.9...v1.3.10

[1.3.11]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.10...v1.3.11

[1.4.0]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.3.11...v1.4.0

[1.4.1]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.0...v1.4.1

[1.4.2]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.1...v1.4.2

[1.4.3]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.2...v1.4.3

[1.4.4]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.3...v1.4.4

[1.4.5]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.4...v1.4.5

[1.4.6]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.5...v1.4.6

[1.5.0]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.4.6...v1.5.0