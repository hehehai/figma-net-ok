# Rust - FigmaNetOK

> look repo: https://github.com/Moonvy/Figma-Net-OK

## usage

> **Warning**
> 如果使用 reset 命令，请务必先备份您的 hosts 文件。

```
git clone https://github.com/hehehai/figma-net-ok.git
cd figma-net-ok

cargo check
sudo cargo run
```

> **Note**
> 为什么使用 sudo? 需要修改系统 hosts 文件（仅在重置或添加时）。

- [x] 重置 Figma 相关 hosts
- [x] 查找最优 ip
- [x] 自动设置最优 ip 到 hosts
- [x] 错误处理
- [x] 测试
- [ ] 打包
