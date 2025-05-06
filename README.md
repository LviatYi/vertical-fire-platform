# Vertical Fire Platform

**垂直火力平台 (Vertical Fire Platform)** 是软化开发工作流的工具集合。

v1.4.1  
by LviatYi

阅读该文档时，推荐安装以下字体：

- [JetBrainsMono Nerd Font
  Mono][JetbrainsMonoNerdFont]
- [Sarasa Mono SC][SarasaMonoSC]

若出现乱码，其为 Nerd Font 的特殊字符，不影响段落语义。

## Change ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

Change Log:

- v1.4.1
    - 修复当多个重复状态的 Run Task 时，无法正确获取最新包的问题。
    - 在构建错误并输出日志前，展示 url 供直接跳转。
- v1.4.0
    - 追加 Build 功能。一键发起 Jenkins Build，包含自动查询默认参数、填充推荐参数、智能记忆以及操作链。
    - 在 fp watch 时主动询问 Job Name。
    - 支持自定义 Job Name.
- v1.3.11
    - 取消了 Cookie 登录的支持。添加了密码登录。
- v1.3.10
    - 添加 watch 功能。它会在 Jenkins 上有新包时，自动下载并解压。
    - 取消 Login 命令时请求 job_name 的输入。现在流程上不再必要。
    - 修复 错误的 build number of last run checking。该错误导致查询用户上次解压包时，若该包不是最新包，则错误推断为不存在。
    - 修复 在输入 job name 时，未根据上次输入对 Jenkins 进行排序。
- v1.3.9
    - 提前存储成功登陆后的 Jenkins 信息。
    - 添加调试模式。
- v1.3.8
    - 优化了当 Jenkins 登录状态疑似过期时的提示。
    - Jenkins 相关功能结构调整。
    - 调整操作历史存储，取消 branch 字段，合并功能为 interest_job_name 字段。
    - 持续模块化输入函数。
    - 添加查询用户最新包时的等待提示文本。
    - 向查询用户最新包添加额外的信息：构建中、构建失败。
    - 优化输出颜色，使其更具表达性。
- v1.3.7
    - 优化 `main.rs` 中的代码。使用模块化的函数代替过程。
    - 修复当 RunInfo 结果为 null 时引发的解析错误。该错误曾导致当有进行中的 Run Task 时，无法获得用户当前的最新已完成任务。
- v1.3.6
    - 添加了与 Jenkins 进行数据沟通的能力。使用 `fp login` 选择一种方式进行登录，随后使用 `fp extract` 时将快速得到专属于你的包。
    - 推荐 Cookie 登录而非 ApiToken。ApiToken 非常的慢！
- v1.3.5
    - 引入了全新的运行存储管理，使得对不同版本的数据具有更好的兼容性与开发可读性。
    - 修复了一些错误的单元测试配置。
- v1.3.4
    - 添加了重置 `fp run -S` 的方式。当用户未输入 `-S` 参数时，将采用 localhost 作为服务器。
    - 修复了模拟 index 的偏移。该修复可能导致原有的账号信息反偏移。
- v1.3.3
    - 使用环境变量配置敏感数据。
    - 优化 pretty log 结构，使其更具表达性。
        - 添加了 VfpPrettyLogger 结构，使用其构造函数替换原本的 `pretty_log_operation_start` 函数。
        - 使 `pretty_log_operation_status` 被调用前必须调用 `pretty_log_operation_start` 成为一种可靠保证。
- v1.3.2
    - 使用 variable.toml 代替敏感数据保存方式。
    - 打通 Github CI 流程。

Road Map:

- [ ] 优化代码结构。
- [ ] 提供一种直接打开记忆存储文件的方式。

## Functional ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

它旨在提供如下便利：

- [x] **应急按钮** 仅使用回车完成常用功能。
- [x] **记忆海绵** 保存操作参数。强配置性，但只处理一遍。
- [x] **多虑后勤** 包揽尽可能多的重复工作。
- [x] **无畏并发** 并发的解压与交互输出。
- [x] **时髦输出** 漂亮的控制台输出。

## Commands ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

在此之前，容许我介绍一个很简单的小技巧。也许你已经知道了。

Windows 通过 Path 环境变量来指定可执行文件的路径。在命令行中，你可以直接输入可执行文件名来运行它。这是因为 Windows 会在
Path 中的路径中查找这个文件。  
如果你希望像示例那样，简单地输入 `fp` 来运行程序，那么你需要程序所在的路径添加到 Path 中。

```shell
// 你可以输入这个命令 将程序所在的路径添加到 Path 环境变量中
setx path "%path%;PATH_TO_FP_ROOT_DIR"
```

当然，你需要自己替换 `PATH_TO_FP_ROOT_DIR`。它应该是一个路径，下面包含 fp.exe.

### Extract

解压包。登录后获得更好的体验。

可以这样使用：

```shell
// usage
fp extract -j dev -ci 1111 -c 4 -d C:/path/to/extract
```

也可以这样使用：

```shell
// usage
fp extract
```

随后将有提示，引导用户填入参数。前者的好处是额外使用了控制台提供的操作保存功能，它允许用户使用方向键选择历史操作。

Extract 提供了以下参数：

- **-j, --job-name <JOB_NAME>** 任务名。
- **-#, --ci <CI>** 包 ID。用于定位包。
- **-c, --count <COUNT>** 期望数量。指定解压数量。
- **--repo <BUILD_TARGET_REPO_TEMPLATE>** [仅调试] 包仓库模板。在这其中搜索包。
- **--locator-pattern <MAIN_LOCATOR_PATTERN>** [仅调试] 主定位器模式。
- **--s-locator-template <SECONDARY_LOCATOR_TEMPLATE>** [仅调试] 次定位器模板。
- **-d, --dest <DEST>** 解压目标路径。

---

### Run

运行实例。

可以这样使用：

```shell
fp run -d C:/path/to/extract -c 4 -p package -e run.exe -k check
```

也可以这样使用：

```shell
fp run
```

- **-d, --dest <DEST>** 解压目标路径。一般与 Extract 的 -d 路径相同。
- **-c, --count-or-index <COUNT_OR_INDEX>** 期望数量。指定启动的个数。
- **-i, --index <INDEX>** 索引。指定启动的索引。当指定时，无视 -c 参数。
- **-p, --package-name <PACKAGE_FILE_STEM>** 包名。用于确定文件夹名称。
- **-e, --exe-name <EXE_FILE_NAME>** 可执行文件名。
- **-k, --check-name <CHECK_EXE_FILE_NAME>** 用于检查实例是否已存在的可执行文件名。
- **-f, --force** 强制启动。若实例已存在则关闭它。
- **-S, --server <SERVER>** 使用指定的服务器。

---

### Login

登录 Jenkins 以获得实时信息支持。

可以这样使用：

```shell
// usage
fp login --url https://your.jenkins.url -u your_username -p your_password

// or
fp login --url https://your.jenkins.url -u your_username -a your_api_token
```

也可以这样使用：

```shell
// usage
fp login
```

- **-u, --username <USERNAME>** 用户名 它可能是个邮箱账号，如 "somebody@email.com"
- **-p, --pwd <PASSWORD>** Password。推荐使用，它比 Api Token 的访问更快。
- **-a, --api-token <API_TOKEN>** API
  token。你可以在此处获得更多信息：https://www.jenkins.io/doc/book/using/remote-access-api/

---

### Build

**[需要登录]** 向 Jenkins 发起一次构建任务。

可以这样使用：

```shell
// usage
fp build -j your_interested_job_name --cl 321 --sl 123,456,789
```

也可以这样使用：

```shell
fp build
```

- **-j, --job-name <JOB_NAME>** 你感兴趣的 Jenkins job name。
- **--cl <CL>** change list。
- **--sl <SL>** shelved change list。用任何非空格字符隔开，推荐 `,`。
- **--param <PARAM_NAME> <PARAM_VALUE>** 参数。使用键值对的方式传入。你可以使用多次。
- **--no-watch-and-extract** 在所需的操作成功后，不要执行监视与自动解压。
- **--no-extract** 在所需的操作成功后，不要执行自动解压。

---

### Watch

**[需要登录]** 监控 Jenkins 平台的 Run task 完成状态，以自动进行进一步操作。

可以这样使用：

```shell
// usage
fp watch -j your_interested_job_name --ci 1111
```

也可以这样使用：

```shell
// usage
fp watch
```

- **-j, --job-name <JOB_NAME>** 你感兴趣的 Jenkins job name。
- **-#, --ci <CI>** 包 ID。用于定位包。
- **--no-extract** 在所需的操作成功后，不要执行自动解压。

---

[JetbrainsMonoNerdFont]: https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.2/JetBrainsMono.zip@fallbackFont

[SarasaMonoSC]: https://github.com/be5invis/Sarasa-Gothic/releases/download/v0.41.6/sarasa-gothic-ttf-0.41.6.7z
