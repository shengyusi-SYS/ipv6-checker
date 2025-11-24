# IPv6 Checker Service

纯 ai 开发，未严格审查，自测可用。一个用于 Windows 的系统服务，用于获取 IPv6 地址。

## 功能特性

- ✅ 读取当前工作目录下的 `config.json` 配置文件
- ✅ 首次运行自动创建默认配置文件
- ✅ 监听指定端口（默认 3443）
- ✅ 并发查询多个 URL，返回最快的结果
- ✅ 可安装为 Windows 系统服务，随系统自动启动
- ✅ 使用静态链接，无需 VC 运行时
- ✅ 所有服务管理功能内置，无需外部脚本

## 配置文件

首次运行会自动在可执行文件同目录创建 `config.json`：

```json
{
  "port": 3443,
  "urls": [
    "https://api64.ipify.org?format=json",
    "https://api6.ipify.org?format=json",
    "https://icanhazip.com"
  ]
}
```

## 编译

```powershell
cargo build --release
```

编译后的可执行文件位于 `target\release\ipv6-checker.exe`

## 快速开始

### 1. 控制台模式（测试用）

直接运行可执行文件：

```powershell
.\ipv6-checker.exe
```

程序会自动创建 `config.json` 并启动 HTTP 服务器。

### 2. 安装为 Windows 服务

**需要管理员权限**

```powershell
# 显示帮助
.\ipv6-checker.exe help

# 安装服务（自动启动类型，使用LocalSystem账户）
.\ipv6-checker.exe install

# 启动服务
.\ipv6-checker.exe start

# 查看服务状态
.\ipv6-checker.exe status
```

## 服务管理命令

所有命令都需要管理员权限（除了 help 和 status）：

```powershell
ipv6-checker.exe install      # 安装服务
ipv6-checker.exe uninstall    # 卸载服务
ipv6-checker.exe start        # 启动服务
ipv6-checker.exe stop         # 停止服务
ipv6-checker.exe restart      # 重启服务
ipv6-checker.exe status       # 查看状态
ipv6-checker.exe help         # 显示帮助
```

## 服务特性

- **自动启动**: 服务配置为随系统自动启动
- **系统账户**: 使用 LocalSystem 账户运行，可在用户登录前启动
- **无需运行时**: 静态链接所有依赖，无需安装 VC 运行时
- **并发查询**: 同时查询所有配置的 URL，返回最快的结果
- **自动配置**: 首次运行自动创建配置文件

## API 端点

### 获取 IPv6 地址

```http
GET http://localhost:3443/
GET http://localhost:3443/ipv6
```

响应示例：

```json
{
  "ipv6": "2001:0db8:85a3::8a2e:370:7334"
}
```

### 健康检查

```http
GET http://localhost:3443/health
```

响应示例：

```json
{
  "status": "ok"
}
```

## 完整安装示例

```powershell
# 以管理员身份打开 PowerShell

# 1. 将程序复制到目标位置（可选）
$installPath = "C:\Program Files\IPv6Checker"
New-Item -ItemType Directory -Path $installPath -Force
Copy-Item .\target\release\ipv6-checker.exe -Destination $installPath

# 2. 进入安装目录
cd $installPath

# 3. 安装并启动服务
.\ipv6-checker.exe install
.\ipv6-checker.exe start

# 4. 检查状态
.\ipv6-checker.exe status

# 5. 测试API
Invoke-RestMethod -Uri http://localhost:3443/ipv6
```

## 卸载服务

```powershell
# 以管理员身份运行
.\ipv6-checker.exe stop
.\ipv6-checker.exe uninstall

# 如需删除程序文件
Remove-Item -Path "C:\Program Files\IPv6Checker" -Recurse -Force
```

## 故障排除

### 服务无法启动

1. 检查 `config.json` 是否在可执行文件同目录
2. 确保端口未被占用
3. 检查防火墙设置
4. 以控制台模式运行查看详细错误信息

### 权限问题

所有服务管理命令需要管理员权限，请以管理员身份运行 PowerShell。

### 测试连接

```powershell
# 测试IPv6端点
curl http://localhost:3443/ipv6

# 或使用 PowerShell
Invoke-RestMethod -Uri http://localhost:3443/ipv6

# 检查健康状态
curl http://localhost:3443/health
```

## 技术细节

- **语言**: Rust
- **Web 框架**: Axum
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **正则表达式**: Regex
- **编译目标**: x86_64-pc-windows-msvc (静态链接)
