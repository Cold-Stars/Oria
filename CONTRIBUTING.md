# 贡献指南

感谢你考虑为 Oria 做出贡献！

## 🌟 贡献方式

### 报告 Bug

如果你发现了 Bug，请创建一个 Issue 并包含以下信息：

- **Bug 描述**：清晰简洁的描述
- **复现步骤**：详细的复现步骤
- **期望行为**：你期望发生什么
- **实际行为**：实际发生了什么
- **截图**：如果适用，添加截图
- **环境信息**：
  - 操作系统及版本
  - Oria 版本
  - 其他相关信息

### 提出功能建议

我们欢迎新功能建议！请创建一个 Issue 并包含：

- **功能描述**：清晰描述建议的功能
- **使用场景**：为什么需要这个功能
- **替代方案**：你考虑过的其他解决方案
- **附加信息**：任何其他有帮助的信息

### 贡献代码

#### 开发环境设置

1. **Fork 仓库**

2. **克隆你的 Fork**
```bash
git clone https://github.com/your-username/oria.git
cd oria
```

3. **安装依赖**
```bash
pnpm install
```

4. **启动开发服务器**
```bash
pnpm tauri dev
```

#### 开发流程

1. **创建分支**
```bash
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/your-bug-fix
```

2. **进行修改**
   - 遵循代码规范
   - 添加必要的测试
   - 更新相关文档

3. **提交代码**
```bash
git add .
git commit -m "feat: add amazing feature"
```

4. **推送到你的 Fork**
```bash
git push origin feature/your-feature-name
```

5. **创建 Pull Request**
   - 在 GitHub 上创建 PR
   - 填写 PR 模板
   - 等待审核

## 📝 代码规范

### 提交信息规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<类型>(<范围>): <描述>

[可选的正文]

[可选的脚注]
```

**类型：**
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响代码运行）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例：**
```bash
feat(canvas): add zoom in/out functionality
fix(export): correct YOLO format export bug
docs(readme): update installation instructions
```

### JavaScript/Vue 规范

- 使用 ESLint 和 Prettier
- 组件使用 Composition API
- 遵循 Vue 3 风格指南

```bash
# 格式化代码
pnpm run format

# 检查代码
pnpm run lint
```

### Rust 规范

- 使用 rustfmt 格式化
- 使用 clippy 检查

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy
```

## 🧪 测试

在提交 PR 前，请确保：

- [ ] 代码通过所有测试
- [ ] 新功能添加了测试
- [ ] 更新了相关文档
- [ ] 代码格式正确

```bash
# 运行测试
cargo test

# 构建项目
pnpm tauri build
```

## 📚 文档

如果你的更改涉及用户可见的功能：

- 更新 README.md
- 更新相关文档
- 添加必要的注释
- 更新 CHANGELOG.md

## 🔍 代码审查

### 审查流程

1. 自动化检查（CI）
2. 代码审查
3. 测试验证
4. 合并

### 审查标准

- 代码质量
- 测试覆盖
- 文档完整性
- 性能影响
- 向后兼容性

## 💡 开发技巧

### 调试

```bash
# 启动开发模式（带调试信息）
RUST_LOG=debug pnpm tauri dev
```

### 热更新

前端代码修改会自动热更新，Rust 代码需要重新编译。

### 工具推荐

- **IDE**: VS Code
- **插件**:
  - Volar (Vue)
  - rust-analyzer (Rust)
  - ESLint
  - Prettier

## 🤝 行为准则

### 我们的承诺

为了营造一个开放和友好的环境，我们承诺：

- 尊重不同的观点和经验
- 接受建设性的批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

### 不可接受的行为

- 使用性暗示的语言或图像
- 人身攻击或侮辱性评论
- 公开或私下的骚扰
- 未经许可发布他人隐私信息
- 其他不专业或不受欢迎的行为

## 📞 联系方式

如有任何问题，欢迎通过以下方式联系：

- 💬 GitHub Discussions
- 🐛 GitHub Issues
- 📧 Email: your-email@example.com

## 🎉 贡献者

感谢所有为 Oria 做出贡献的人！

<!-- ALL-CONTRIBUTORS-LIST:START -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

---

再次感谢你的贡献！🎉

