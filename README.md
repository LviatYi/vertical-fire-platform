# Vertical Fire Platform

**垂直火力平台 (Vertical Fire Platform)** 是软化开发工作流的工具集合。

v1.3.2  
by LviatYi

阅读该文档时，推荐安装以下字体：

- [JetBrainsMono Nerd Font
  Mono][JetbrainsMonoNerdFont]
- [Sarasa Mono SC][SarasaMonoSC]

若出现乱码，其为 Nerd Font 的特殊字符，不影响段落语义。

## Change ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

Change Log:

- v1.3.3*
    - 优化 pretty log 结构，使其更具表达性。
        - 添加了 VfpPrettyLogger 结构，使用其构造函数替换原本的 `pretty_log_operation_start` 函数。
        - 使 `pretty_log_operation_status` 被调用前必须调用 `pretty_log_operation_start` 成为一种可靠保证。
- v1.3.2
    - 使用 variable.toml 代替敏感数据保存方式。
    - 打通 Github CI 流程。

Road Map:

- 目前没有额外计划。

## Functional ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

它旨在提供如下便利：

- [x] **应急按钮** 仅使用回车完成常用功能。
- [x] **记忆海绵** 保存操作参数。强配置性，但只处理一遍。
- [x] **多虑后勤** 包揽尽可能多的重复工作。
- [x] **时髦输出** 漂亮的控制台输出。

## Commands ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

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

[JetbrainsMonoNerdFont]: https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.2/JetBrainsMono.zip@fallbackFont
[SarasaMonoSC]: https://github.com/be5invis/Sarasa-Gothic/releases/download/v0.41.6/sarasa-gothic-ttf-0.41.6.7z
