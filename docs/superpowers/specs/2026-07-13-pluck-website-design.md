# Pluck 官网设计方案

日期：2026-07-13

## 1. 背景与目标

Pluck 当前通过 GitHub README 和 Releases 对外展示，缺少一个能完整说明产品定位、工作流和下载方式的官网。第一版官网部署到 GitHub Pages，使用 `https://hzpeng57.github.io/pluck/`，同时提供英文与简体中文内容。

官网需要完成三个目标，优先级从高到低：

1. 让目标用户快速理解 Pluck 是什么，以及它与 JetBrains IDE Git 工作流之间的关系。
2. 引导 Apple Silicon 用户下载最新版本。
3. 将希望了解实现或参与项目的用户引导到 GitHub 仓库。

首要转化是下载最新版，GitHub 是次要转化。

## 2. 目标用户与核心定位

目标用户是熟悉 WebStorm、IntelliJ IDEA、PyCharm 等 JetBrains IDE Git 操作，希望在独立 macOS 应用中保留类似操作习惯的开发者。

英文核心表达：

> Pluck. Git with developer muscle memory.

> A focused macOS Git client, inspired by the Git workflows in JetBrains IDEs.

中文核心表达：

> Pluck，让熟悉的 Git 操作成为肌肉记忆。

> 一款专注于 macOS 的 Git 客户端，操作方式参考 JetBrains 系列 IDE 的 Git 工作流。

文案使用 “inspired by” 或“参考”，不将 Pluck 描述成 JetBrains 产品或官方扩展。页脚明确说明 Pluck 是独立项目，与 JetBrains 没有隶属、授权或赞助关系。

## 3. 范围与非目标

### 3.1 第一版范围

- 中英文双语单页产品站。
- 真实产品截图和工作流展示。
- 最新版本下载入口和 GitHub 入口。
- Apple Silicon、alpha 状态、未签名安装步骤说明。
- 完整的基础 SEO、社交分享元数据和结构化数据。
- 桌面、平板和移动端响应式布局。
- GitHub Pages 自动部署。
- README 和 GitHub 仓库主页增加官网入口。

### 3.2 第一版不做

- 博客、文档系统和在线 changelog。
- 用户账户、服务端 API 或数据库。
- 在线自动检测本机架构。
- 网站分析和第三方追踪脚本。
- 虚构的用户评价、下载量、团队规模或商业指标。
- Intel Mac 和 Windows 的不可用下载入口。

## 4. 技术架构

采用“同仓库、独立官网入口”的结构。官网放在 `website/`，与 Tauri 应用源码隔离，但复用仓库现有 Vite、TypeScript 和包管理工具。

目标目录：

```text
website/
  index.html                 英文页面
  zh-CN/index.html           简体中文页面
  404.html                   GitHub Pages 兜底页
  src/
    main.ts                  移动导航、轻量动效等渐进增强
    styles.css               官网独立样式和设计 token
    assets/                  经 Vite 处理的产品图片
  public/
    favicon.*
    og/pluck-cover.png
    robots.txt
    sitemap.xml
  vite.config.ts             /pluck/ base 与多页面构建配置
.github/workflows/pages.yml  GitHub Pages 构建和发布
```

官网采用 HTML-first 的 Vite 多页面构建，不实现客户端路由，也不依赖 JavaScript 才能呈现正文。英文和中文页面各自包含完整语义化 HTML。语言切换使用普通链接，关闭 JavaScript 后仍能正常阅读、切换语言和下载。

根 `package.json` 增加 `website:dev`、`website:build` 和 `website:preview` 三个独立命令。现有 `pnpm dev`、`pnpm build` 和 Tauri 构建行为保持不变。

页面不在运行时调用 GitHub API。下载按钮直接指向 `https://github.com/hzpeng57/pluck/releases/latest`，避免速率限制、加载失败和首屏抖动。构建时可从根 `package.json` 读取当前版本，用于页面可见信息和 JSON-LD。

## 5. 信息架构

### 5.1 顶部导航

- Pluck 图标和品牌名。
- Features / 功能。
- Workflow / 工作流。
- Download / 下载。
- GitHub 外链。
- English / 简体中文切换。

桌面端使用紧凑的吸顶导航；移动端使用图标按钮打开自绘菜单。导航不使用大型胶囊容器，不遮挡正文，并处理 macOS 和 iPhone 的安全区域。

### 5.2 首屏 Hero

- H1 只使用产品名 `Pluck`，保证品牌是第一视口的明确视觉信号。
- 副标题表达“保留开发者操作习惯的 Git 客户端”。
- 一句话说明参考 JetBrains IDE Git 工作流。
- 主按钮：Download for Apple Silicon / 下载 Apple Silicon 版本。
- 次按钮：View on GitHub / 在 GitHub 查看。
- 展示真实 Pluck 主界面，优先呈现提交图谱、分支列表和 diff 上下文。
- 首屏高度控制在每种常见桌面和移动视口都能看到下一节提示，不做铺满多屏的巨型 Hero。

### 5.3 Workflow 工作流

用真实应用截图和简短文案依次展示：

1. Visual history：分支选择、提交图谱、搜索、范围选择和提交详情。
2. Review-ready diff：unified / side-by-side、忽略空白、文件导航和文件回滚。
3. Interactive rebase：reorder、squash、edit、reword，以及冲突中态反馈。
4. Safe branch actions：pull with rebase、merge、cherry-pick、reset、revert 和 force-with-lease。

页面使用全宽内容带和不对称编排，不把每个 section 包装成悬浮卡片。只有重复功能项可以使用边界清晰的小卡片。

### 5.4 产品原则

三条核心信息：

- Familiar by design：延续 JetBrains 用户熟悉的工作流和上下文菜单思路。
- Git underneath：后端直接调用 Git CLI，展示真实仓库状态，不建立另一套仓库模型。
- Focused on the work：在一个工作台内完成查看、审阅、提交和历史操作。

“Git underneath”只描述技术事实，不暗示所有 CLI 能力都已在 UI 中实现。

### 5.5 下载与安装

- 明确当前支持 macOS Apple Silicon。
- 明确项目仍处于 personal-use alpha。
- 下载按钮进入 GitHub latest release。
- 展示 DMG 安装步骤。
- 说明当前未通过 Apple Developer ID 签名，并提供一次性的 `xattr -dr com.apple.quarantine /Applications/Pluck.app` 命令。
- 说明正式签名和公证在规划中。

### 5.6 开源区域与页脚

- GitHub repository、Releases 和 License 链接。
- 当前版本或 latest release 链接。
- JetBrains 独立项目免责声明。
- 中英文页面互链。

## 6. 视觉设计

采用已确认的 **Developer Energy** 方向。

### 6.1 色彩

- 主背景延续应用的深色中性色，如 `#0d0e10` 和 `#141615`。
- 主强调色使用 Pluck 的薄荷绿 `#78d7bd`。
- Git 图谱和功能状态少量使用蓝、粉、黄、绿色，来源与应用现有 graph token 保持一致。
- 渐变只允许出现在局部品牌字、图谱轨迹或细节高光，不使用大面积紫蓝渐变背景。
- 不使用装饰性光球、bokeh 或与产品无关的抽象背景。

### 6.2 排版

- 正文优先使用 macOS 系统字体栈，避免首屏等待远程字体。
- hash、branch、命令和状态信息使用系统等宽字体。
- Hero 标题只服务于真实首屏，不把同等字号带入功能面板。
- 不使用随 viewport 宽度线性缩放的字体；通过明确断点和 `clamp()` 的尺寸上下限控制版式。
- letter-spacing 保持 `0`，不用负字距制造紧凑感。

### 6.3 图像

- 主视觉必须使用真实 Pluck 应用截图，不使用虚构产品界面。
- 截图前创建或使用脱敏的示例 Git 仓库，提交信息和文件内容不得暴露私人仓库、用户名或内部项目。
- 至少准备一张完整工作台截图，以及 log、diff、rebase 三类局部截图。
- 输出 AVIF 和 WebP，保留 PNG 兜底；提供固定宽高和响应式 `srcset`。
- 社交分享图为 1200x630，包含 Pluck 图标、产品名、核心定位和可辨认的真实界面。

### 6.4 动效

- 首屏可使用一次性的内容进入和 Git 图谱线条绘制。
- 功能区只做轻量滚动显现，不使用持续漂浮、过度视差或自动轮播。
- 所有动效支持 `prefers-reduced-motion: reduce`，禁用后不影响内容顺序和理解。
- hover 不改变固定组件尺寸，避免布局跳动。

## 7. 双语策略

- 英文为默认语言，根路径输出英文。
- 简体中文使用 `/zh-CN/` 独立路径。
- 两个页面拥有相同的信息结构和功能入口，文案允许自然本地化，不做逐字直译。
- 每个语言页使用正确的 `<html lang>`、标题、描述、图片 alt 和结构化数据。
- 语言切换使用直接链接，不依赖 localStorage 或浏览器语言强制重定向。
- 可以读取浏览器语言提供一次非阻塞提示，但第一版不自动跳转，避免搜索引擎和用户被错误重定向。

## 8. SEO 设计

### 8.1 页面元数据

英文建议标题：

> Pluck - A focused Git client for macOS

中文建议标题：

> Pluck - 专注于 macOS 的 Git 客户端

每个语言页提供独立的：

- `title` 和 `meta description`。
- Open Graph 标题、描述、类型、URL、图片和 locale。
- Twitter Card 元数据。
- 自引用 canonical。
- `hreflang="en"`、`hreflang="zh-CN"` 和 `hreflang="x-default"`。

canonical 和 Open Graph URL 使用完整生产地址，不根据本地 preview 地址动态生成。

### 8.2 结构化数据

每个语言页输出 `SoftwareApplication` JSON-LD，至少包含：

- `name`、`description`、`url` 和 `downloadUrl`。
- `applicationCategory: DeveloperApplication`。
- `operatingSystem: macOS`。
- 当前 `softwareVersion`。
- `isAccessibleForFree: true`。
- GitHub 仓库 `sameAs`。

不提供尚不存在的评分、评论、价格或组织信息。

### 8.3 可抓取性与站点文件

- `robots.txt` 允许抓取，并声明绝对 sitemap URL。
- `sitemap.xml` 同时列出英文和中文 URL，并包含语言 alternate。
- 页面正文、导航和下载链接都存在于初始 HTML。
- 使用语义化 `header`、`nav`、`main`、`section` 和 `footer`。
- 每页只有一个 H1，标题层级连续。
- 404 页面提供返回英文、中文首页和 GitHub 的真实链接。

### 8.4 仓库外部入口

- README 顶部增加官网链接。
- GitHub 仓库 Homepage 设置为生产地址。
- Release 和 README 中保持产品名称、描述与官网一致。

仓库 Homepage 设置属于 GitHub 仓库配置；代码实现完成后单独执行或由用户在 GitHub 设置中完成，不影响 Pages 部署。

## 9. 响应式、可访问性与性能

### 9.1 响应式

- 重点检查 375、768、1280 和 1440 像素宽度。
- 固定格式的应用截图使用稳定的 `aspect-ratio` 和宽度约束。
- 移动端将产品截图保持可辨认，不通过过度裁切隐藏核心界面。
- 按钮文案允许换行，但图标、文字和点击区域不能重叠。

### 9.2 可访问性

- 颜色对比达到 WCAG AA。
- 所有交互可键盘访问，并有清晰 `:focus-visible`。
- 图标按钮提供可访问名称和 tooltip。
- 菜单打开后管理焦点，Escape 可关闭。
- 装饰性图像使用空 alt，产品截图使用说明性 alt。
- 页面在 200% 缩放下保持可用。

### 9.3 性能

- 不引入重量级动画库、轮播库或客户端路由。
- 首屏关键 CSS 与脚本保持小体积，脚本延迟执行。
- 首屏图片设置合理的 preload 或 `fetchpriority="high"`，其余图片 lazy-load。
- 图片声明 `width`、`height` 和 `aspect-ratio`，降低 CLS。
- 不依赖第三方字体、分析脚本和运行时 GitHub API。

## 10. 发布流程

新增独立 GitHub Pages workflow：

1. 在 `main` 上检测官网、版本或 workflow 相关变更。
2. 安装 pnpm 和 Node。
3. 安装依赖并执行官网构建。
4. 使用 GitHub Pages 官方 actions 配置并上传静态产物。
5. 部署到 `github-pages` environment。

workflow 同时支持 `workflow_dispatch`。Pages workflow 不修改现有 release workflow，也不参与 Tauri release artifact 构建。

如果 GitHub 仓库尚未将 Pages Source 设置为 GitHub Actions，需要在首次部署前完成一次仓库设置。该操作属于远程写操作，执行前单独征得用户确认。

## 11. 错误处理与降级

- GitHub latest release 链接是普通外链；GitHub 临时不可用时，页面其他内容仍正常显示。
- JavaScript 加载失败时，正文、语言切换、下载和 GitHub 链接仍可用。
- 图片加载失败时保留准确 alt 和稳定布局，不让文案区域塌陷。
- 不支持的平台仍可查看官网，但下载区不会误称其设备受支持。
- 无效路径进入自定义 404 页面，不依赖 SPA fallback。

## 12. 验收标准

### 12.1 功能

- `/pluck/` 显示完整英文页面。
- `/pluck/zh-CN/` 显示完整中文页面。
- 两页可相互切换，直接访问和刷新都成功。
- 下载、GitHub、Release 和 License 链接正确。
- 移动导航、键盘操作和 reduced motion 正常。
- 根应用的 `pnpm build` 和 Tauri 配置不受官网入口影响。

### 12.2 SEO

- 两个页面在禁用 JavaScript时仍包含完整正文。
- title、description、canonical、hreflang、Open Graph 和 Twitter Card 正确。
- JSON-LD 可被结构化数据工具解析，内容不包含虚假字段。
- `robots.txt`、`sitemap.xml` 和 404 页面可访问。
- 所有生产绝对 URL 都包含 `/pluck/` 子路径。

### 12.3 视觉与质量

- 使用真实、脱敏的 Pluck 产品截图。
- 375、768、1280 和 1440 宽度没有溢出、遮挡或重叠。
- 页面在桌面和移动端第一视口都能看到下一节内容提示。
- 关键文本和控件满足 WCAG AA 对比度。
- Lighthouse 桌面端 Performance、Accessibility 和 Best Practices 不低于 95，SEO 为 100；移动端 Performance 不低于 90，其余三项沿用桌面端阈值。
- 使用 Playwright 截图检查桌面与移动端视觉，并验证图片非空、资源无 404、按钮可点击。

## 13. 实施边界

官网开发只修改 `website/`、官网构建脚本、Pages workflow、README 和必要的仓库元数据，不重构 Pluck 应用本身。截图准备过程中如果发现应用 UI 问题，只记录，不顺带修改应用代码。

实现阶段结束后先提供本地预览地址和验证结果。任何远程 push、GitHub Pages 设置或仓库 Homepage 修改都必须在执行前获得用户明确确认。
