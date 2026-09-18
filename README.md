# 小团子桌宠

仅支持 Windows 的本地桌宠。使用 Tauri 2、Rust、Preact、TypeScript 和 Canvas 2D，不连接 AI 或任何网络服务。

## 开发运行

```powershell
npm install
npm run tauri dev
```

## 本地构建

```powershell
npm run tauri build -- --no-bundle
```

状态快照保存在系统应用数据目录。桌面启动器直接启动本地编译后的程序。

## 当前范围

已实现透明无边框窗口、置顶、托盘、独立设置窗口、四档尺寸、Canvas 2D 角色、点击/拖动反馈、气泡、Rust 权威状态、JSON 快照、单实例与托盘退出。

尚未完成透明区域的逐像素点击穿透、多显示器热插拔修正、全屏自动隐藏、开机启动、睡眠唤醒恢复和签名安装包。
