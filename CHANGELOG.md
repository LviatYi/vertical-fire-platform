# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [1.7.0] - 2026-03-19

## [1.6.10] - 2026-02-12

### 🚀 Added

- 通过 json api 获取 Jenkins Job 的参数信息，以绕过 config.xml 接口的权限问题。

## [1.6.9] - 2025-09-11

### ⚙️ Changed

- 优化 `fp update` 的交互反馈。

### 🐛 Fixed

- 将 self update 置入 block 运行时。

### ✅ Test

- 移除或忽略了一些不必要的测试。

## [1.6.8] - 2025-09-04

### ⚙️ Changed

- 添加新的 Quit 错误类型，以便未来进行功能添加，从而在用户中断操作时，优雅地退出程序。
- 逐步支持快速中断操作。目前支持 `fp build` 以及在输入 `job_name` `ci` 时的快速中断。由于快速中断的处理意味着原有的无异常变为有异常，因此改造时需要谨慎进行，还望理解。
- 添加了一些调试代码，以跟踪 `fp watch` 时，无法得到最新完成包所使用 CL 的问题。

## [1.6.7] - 2025-08-25

### 🐛 Fixed

- 在 `query_user_latest_info` 中引入更明确的语义。该问题曾导致 `fp watch` 及其关联命令异常。

## [1.6.6] - 2025-08-22

### ⚙️ Changed

- 优化 CL 输入错误时的提示。

### 🐛 Fixed

- 修复了 `query_user_latest_info` 中错误的 `processing` 与 `failed` 记录。该问题曾导致 `fp watch` 及其关联命令异常。

## [1.6.5] - 2025-08-21

### 🚀 Added

- 并行查询用户最新的 Run Task 信息，极大加快了查询速度。
- 添加 `JenkinsRpcService` 为未来的同一管理 RPC 调用服务进行铺垫。

## [1.6.4] - 2025-08-18

此次更新旨在利用正则表达式优化参数传递体验！

### 🚀 Added

- 对 `fp extract`、`fp watch` 添加 `-u` 参数的支持，现在可以通过 `-u` 或 `--url` 直接传入 URL，藉此解析 job_name 与 ci。
- 可通过 `fp db` 命令，直接访问记忆文件。这是个危险的举动，如果改动，你可能会丢失数据。请提前做好备份~

## [1.6.3] - 2025-08-14

### 🚀 Added

- 新增 AppState 进行全局状态管理。

### ⚙️ Changed

- 优化 StdOut 取用模式。
- 优化 DbDataProxy 取用模式，避免未来开发中竞态读写问题。

### 🐛 Fixed

- 修复了 RepoDecoration 中，未使用 default_config 默认值填充空值的问题。该问题曾导致首次使用 fp 时，未能定位到远程文件仓库。

## [1.6.2] - 2025-08-08

### 🚀 Added

- 新增了对包含 Choice 参数的 Jenkins Job 的支持。
    - 现在可以在 `fp build` 时，自动获取并填充 Choice 参数。

### ⚙️ Changed

- 优化代码结构。

### ✅ Test

- 维护了针对 JobConfig 的单元测试。

## [1.6.1] - 2025-08-05

### ⚙️ Changed

- 保持记忆文件中，job 相关的存储项目上限为 8 个。

## [1.6.0] - 2025-08-05

**[重大更新]** 此次更新旨在优化分支切换体验！

### 🚀 Added

- 调整记忆存储，根据 `job_name` 快速切换分支相关的偏好设置。

### ⚙️ Changed

- 优化更新流程。
- 允许无法查询 config.xml 时，仍然可以使用 `fp build`。
    - 将自动传入所有缓存参数，其他则需要手动传入。
- 修改 job_name 的输入方式，以参考记忆进行提示。

### ✅ Test

- 补充了一些单元测试用例。
- 实践了一种新的测试范式：使用 `#[cfg(debug_assertions)]` 属性嵌入 debug 环境中的测试代码。

## [1.5.4] - 2025-07-29

### 🚀 Added

- 尝试在获取 CL 失败后，重试获取。

### ⚙️ Changed

- 优化获取 CL 失败后的日志。

### 🐛 Fixed

- 当 `fp build` 未填写 Cl 需要自动获取信息时，使用了错误的谓词。

## [1.5.3] - 2025-07-25

### ⚙️ Changed

- 优化版本更新提示。

### 🐛 Fixed

- 启用 `compression-zip-deflate` 功能，以修复 self_update 无法解压的问题。

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

[Unreleased]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.7.0...HEAD

[1.7.0]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.10...v1.7.0

[1.6.10]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.9...v1.6.10

[1.6.9]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.8...v1.6.9

[1.6.8]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.7...v1.6.8

[1.6.7]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.6...v1.6.7

[1.6.6]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.5...v1.6.6

[1.6.5]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.4...v1.6.5

[1.6.4]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.3...v1.6.4

[1.6.3]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.2...v1.6.3

[1.6.2]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.1...v1.6.2

[1.6.1]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.6.0...v1.6.1

[1.6.0]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.5.4...v1.6.0

[1.5.4]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.5.3...v1.5.4

[1.5.3]: https://github.com/LviatYi/vertical-fire-platform/compare/v1.5.2...v1.5.3

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