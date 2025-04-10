# Vertical Fire Platform

**垂直火力平台 (Vertical Fire Platform)** 是软化开发工作流的工具集合。

v1.3.7  
by LviatYi

阅读该文档时，推荐安装以下字体：

- [JetBrainsMono Nerd Font
  Mono][JetbrainsMonoNerdFont]
- [Sarasa Mono SC][SarasaMonoSC]

若出现乱码，其为 Nerd Font 的特殊字符，不影响段落语义。

## Change ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

Change Log:

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

- 添加 Rebuild 功能。
- 添加使用密码而非手动复制 Cookie 进行登录的方式。

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

解压包。

可以这样使用：

```shell
// usage
fp extract -b dev -ci 1111 -c 4 -d C:/path/to/extract
```

也可以这样使用：

```shell
// usage
fp extract
```

随后将有提示，引导用户填入参数。前者的好处是额外使用了控制台提供的操作保存功能，它允许用户使用方向键选择历史操作。

Extract 提供了以下参数：

- **-b, --branch <BRANCH>** 分支名。
- **-#, --ci <CI>** 包 ID。用于定位包。
- **-c, --count <COUNT>** 期望数量。指定解压数量。
- **--repo <BUILD_TARGET_REPO_TEMPLATE>** [仅调试] 包仓库模板。在这其中搜索包。
- **--locator-pattern <MAIN_LOCATOR_PATTERN>** [仅调试] 主定位器模式。
- **--s-locator-template <SECONDARY_LOCATOR_TEMPLATE>** [仅调试] 次定位器模板。
- **-d, --dest <DEST>** 解压目标路径。
- **-r, --reset** 清除操作历史缓存。

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
- **-c, --count-or-index <COUNT_OR_INDEX>** 期望数量或索引。指定启动的个数或索引，具体类型取决于 -s 参数。
- **-p, --package-name <PACKAGE_FILE_STEM>** 包名。用于确定文件夹名称。
- **-e, --exe-name <EXE_FILE_NAME>** 可执行文件名。
- **-k, --check-name <CHECK_EXE_FILE_NAME>** 用于检查实例是否已存在的可执行文件名。
- **-s, --single** 是否单个启动。否则多启动。单个启动意味着 -c 参数为索引。
- **-f, --force** 强制启动。若实例已存在则关闭它。
- **-S, --server <SERVER>** 使用指定的服务器。

### Login

登录 Jenkins 以获得实时信息支持。

可以这样使用：

```shell
// usage
fp login --url https://your.jenkins.url -u your_username -a your_api_token -j your_interested_job_name

// or
fp login --url https://your.jenkins.url -u your_username -c your_cookie -j your_interested_job_name
```

也可以这样使用：

```shell
// usage
fp login
```

- **-u, --username <USERNAME>** 用户名 它可能是个邮箱账号，如 "somebody@email.com"
- **-a, --api-token <API_TOKEN>** API
  token。你可以在此处获得更多信息：https://www.jenkins.io/doc/book/using/remote-access-api/
- **-c, --cookie <COOKIE>** Cookie。它不是很安全，但它在我的用例中更快。如果你不知道哪里可以找到 Cookie，请不要使用。
- **-j, --job-name <JOB_NAME>** 你感兴趣的 Jenkins job name。

---

[JetbrainsMonoNerdFont]: https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.2/JetBrainsMono.zip@fallbackFont

[SarasaMonoSC]: https://github.com/be5invis/Sarasa-Gothic/releases/download/v0.41.6/sarasa-gothic-ttf-0.41.6.7z
