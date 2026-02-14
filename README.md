# ClipShare

LAN 内的分布式剪贴板同步工具。

## 动机

[lan-mouse](https://github.com/feschber/lan-mouse) 实现了局域网内多台机器共享鼠标和键盘，但没有内置剪贴板支持——在机器 A 复制的内容无法直接粘贴到机器 B。

现有方案要么依赖完整的远程桌面协议（如 Barrier/Synergy），要么需要手动操作（如 KDE Connect 的剪贴板同步仅限 Android）。ClipShare 用最简单的方式填补这个空缺：

- **上传自动**：daemon 后台监控剪贴板变化，检测到新内容立即 POST 到 server
- **粘贴手动**：浏览器打开 Web UI，从列表中点击想要的条目写入本地剪贴板

这样每台机器只需运行一个 daemon 进程，打开浏览器就能在任意设备间传递文本、HTML 和图片。

## 快速开始

```bash
# 安装依赖
pixi install

# 启动 server（首次自动生成自签证书）
pixi run server

# 另一个终端，启动 daemon
pixi run daemon

# 浏览器访问 https://localhost:8443
```

多机使用时，编辑 `~/.config/clipshare/config.toml`：

```toml
[daemon]
server_url = "https://192.168.1.100:8443"
verify_ssl = false
```

## 架构

```
Machine A                  Server                     Machine B
┌───────────┐        ┌──────────────────┐        ┌───────────┐
│  daemon   │─POST──→│  FastAPI + HTTPS  │←─POST──│  daemon   │
│ 监控剪贴板 │        │  SQLite 存储      │        │ 监控剪贴板 │
└───────────┘        │  WebSocket 推送   │        └───────────┘
                     │                  │
┌───────────┐        │                  │        ┌───────────┐
│  浏览器   │←──WS───│  GET /static     │───WS──→│  浏览器   │
│ 查看+粘贴  │        └──────────────────┘        │ 查看+粘贴  │
└───────────┘                                    └───────────┘
```

- **daemon**: 轮询本地剪贴板 → SHA256 去重 → POST 新内容到 server
- **server**: FastAPI (HTTPS) 存储条目、WebSocket 广播、托管 Web UI
- **浏览器**: 实时列表展示，点击条目写入本地剪贴板

支持 `text/plain`、`text/html`、`image/png` 三种内容类型。剪贴板后端自动检测 Wayland (wl-paste) / X11 (xclip) / Windows (win32clipboard)。
