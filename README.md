# Vertical Fire Platform

**垂直火力平台 (Vertical Fire Platform)** 是软化开发工作流的工具集合。

v1.6.8  
by LviatYi

阅读该文档时，推荐安装以下字体：

- [JetBrainsMono Nerd Font
  Mono][JetbrainsMonoNerdFont]
- [Sarasa Mono SC][SarasaMonoSC]

若出现乱码，其为 Nerd Font 的特殊字符，不影响段落语义。

## Change ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

[变更日志 | ChangeLog](./CHANGELOG.md)

更新计划 | Road Map:

- [ ] 添加 `fp info`，以允许查询 Jenkins Build Task 的状态。
- [ ] 某些环境中，可能不存在 wmic 命令，因而无法查询特定可执行文件的运行状态。因此需要额外的替代方案。
- [ ] **持续** 优化代码结构。
- [ ] **持续** 优化提示可读性。

## Functional ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

它旨在提供如下便利：

- [x] **应急按钮** 仅使用回车完成常用功能。
- [x] **记忆海绵** 保存操作参数。强配置性，但只处理一遍。
- [x] **多虑后勤** 包揽尽可能多的重复工作。
- [x] **无畏并发** 并发的解压与交互输出。
- [x] **时髦输出** 漂亮的控制台输出。
- [x] **自我迭代** 提供伟大的自我更新能力。

## Install ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

手动下载并解压：[最新版本](https://github.com/LviatYi/vertical-fire-platform/releases/download/v1.6.8/vfp-v1.6.8-x86_64-pc-windows-msvc.zip)

解压 fp.exe 到任意目录即可。可在该目录下运行 cmd，通过命令行运行。

此外，容许我介绍一个很简单的小技巧。也许你已经知道了。

Windows 通过 Path 环境变量来指定可执行文件的路径。在命令行中，你可以直接输入可执行文件名来运行它。这是因为 Windows 会在
Path 中的路径中查找这个文件。  
如果你希望像示例那样，简单地输入 `fp` 来运行程序，那么你需要程序所在的路径添加到 Path 中。

你可以输入这个命令 将程序所在的路径添加到 Path 环境变量中

```shell
setx path "%path%;PATH_TO_FP_ROOT_DIR"
```

当然，你需要自己替换 `PATH_TO_FP_ROOT_DIR`。它应该是一个路径，下面包含 fp.exe.

## Commands ⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠐⠒⠒⠒⠒⠚⠛⣿⡟⠄⠄⢠⠄⠄⠄⡄⠄⠄⣠⡶⠶⣶⠶⠶⠂⣠⣶⣶⠂⠄⣸⡿⠄⠄⢀⣿⠇⠄⣰⡿⣠⡾⠋⠄⣼⡟⠄⣠⡾⠋⣾⠏⠄⢰⣿⠁⠄⠄⣾⡏⠄⠠⠿⠿⠋⠠⠶⠶⠿⠶⠾⠋⠄⠽⠟⠄⠄⠄⠃⠄⠄⣼⣿⣤⡤⠤⠤⠤⠤⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄⠄

总的来说，在任何场景下，你都可以在命令末尾添加 `-h` 或 `--help` 来获取帮助信息。

### Extract

解压包。登录后获得更好的体验。

可以这样使用：

```shell
fp extract -j dev -ci 1111 -c 4 -d C:/path/to/extract
```

也可以这样使用：

```shell
fp extract
```

随后将有提示，引导用户填入参数。前者的好处是额外使用了控制台提供的操作保存功能，它允许用户使用方向键选择历史操作。

Extract 提供了以下参数：

- **-j, --job-name <JOB_NAME>** 任务名。
- **-#, --ci <CI>** 包 ID。用于定位包。
- **-c, --count <COUNT>** 期望数量。指定解压数量。
- **-d, --dest <DEST>** 解压目标路径。
- **-u, --url <URL>** Jenkins Run Task 全称 URL。可以自动解析 **-j** 与 **-#**，但具有更低的优先级。
- **--repo <BUILD_TARGET_REPO_TEMPLATE>** [仅调试] 包仓库模板。在这其中搜索包。
- **--locator-pattern <MAIN_LOCATOR_PATTERN>** [仅调试] 主定位器模式。
- **--s-locator-template <SECONDARY_LOCATOR_TEMPLATE>** [仅调试] 次定位器模板。

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
fp login --url https://your.jenkins.url -u your_username -p your_password

// or
fp login --url https://your.jenkins.url -u your_username -a your_api_token
```

也可以这样使用：

```shell
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

此外，若执行 extract ，则可以额外使用 `fp extract` 的所有参数。

---

### Watch

**[需要登录]** 监控 Jenkins 平台的 Run task 完成状态，以自动进行进一步操作。

可以这样使用：

```shell
fp watch -j your_interested_job_name --ci 1111
```

也可以这样使用：

```shell
fp watch
```

- **-j, --job-name <JOB_NAME>** 你感兴趣的 Jenkins job name。
- **-#, --ci <CI>** 包 ID。用于定位包。
- **-u, --url <URL>** Jenkins Run Task 全称 URL。可以自动解析 **-j** 与 **-#**，但具有更低的优先级。
- **--no-extract** 在所需的操作成功后，不要执行自动解压。

此外，若执行 extract ，则可以额外使用 `fp extract` 的所有参数。

---

### Update

进行自我更新，或配置自动更新可用性。

可以这样使用，以进行自我更新：

```shell
fp update
```

- **--auto-update** 启用自动更新。在运行结束时自动检查更新并安装。
- **--no-auto-update** 禁用自动更新。在运行结束时自动检查更新。
- **--never-check** 禁用更新检查与所有更新提示。将在下次尝试更新时重新启用。
- **-v, --version <VERSION>** 指定更新版本。默认为最新版本。

---

[JetbrainsMonoNerdFont]: https://github.com/ryanoasis/nerd-fonts/releases/download/v3.0.2/JetBrainsMono.zip@fallbackFont

[SarasaMonoSC]: https://github.com/be5invis/Sarasa-Gothic/releases/download/v0.41.6/sarasa-gothic-ttf-0.41.6.7z
