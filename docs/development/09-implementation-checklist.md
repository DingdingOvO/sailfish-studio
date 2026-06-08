# 09 - 实际开发文档

文档版本：1.0-Beta.2
对应需求：docs/requirements/
对应设计：docs/design/
目标读者：开发者

---

## 目录

1. 环境准备
2. Phase 1: 组织与仓库搭建
3. Phase 2: 核心引擎 Rust 重写
   - 3.1 sf-vm 数据模型与解析
   - 3.2 sf-vm 编译器
   - 3.3 sf-vm 执行器
   - 3.4 sf-vm 扩展系统
   - 3.5 sf-vm 设置引擎
   - 3.6 sf-vm WASM 导出
   - 3.7 sf-blocks 积木编辑器
   - 3.8 sf-renderer 舞台渲染器
   - 3.9 sf-parser 解析器
4. Phase 3: 编辑器 UI 框架
5. Phase 4: 多人协作系统
6. Phase 5: 运行时与打包器
7. Phase 6: 桌面端与移动端
8. Phase 7: 扩展市场与生态
9. Phase 8: 测试与发布

---

## 1. 环境准备

### 1.1 系统要求

- macOS 12+ / Windows 10+ / Ubuntu 20.04+
- 内存：16GB+
- 磁盘：50GB+ 可用空间

### 1.2 安装 Rust (≥ 1.87，edition 2024)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup target add wasm32-unknown-unknown
rustc --version  # 验证 ≥ 1.87
```

### 1.3 安装 Node.js 与 pnpm

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
nvm install 24
nvm use 24
corepack enable pnpm
corepack prepare pnpm@latest --activate
```

### 1.4 安装 Rust 工具链

```bash
cargo install wasm-pack cargo-nextest cargo-audit cargo-tarpaulin cargo-fuzz wasm-bindgen-cli
```

### 1.5 安装前端工具

```bash
pnpm add -g @biomejs/biome @playwright/test
```

### 1.6 安装 Emscripten（C/C++ 扩展编译）

```bash
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk && ./emsdk install 4.2 && ./emsdk activate 4.2
source ./emsdk_env.sh
```

### 1.7 安装 Tauri CLI

```bash
cargo install tauri-cli
```

---

## 2. Phase 1: 组织与仓库搭建

### 2.1 创建 GitHub 组织

1. ✅ 访问 github.com，创建组织 Sailfish-Studio
2. ✅ 上传头像，填写描述
3. ⚠️ 开启双因素认证强制要求（需在 GitHub Web 界面手动开启，Free 计划组织 API 不支持）
4. ✅ 设置基础成员权限为 Read
5. ✅ 添加分支保护规则（main 分支要求 PR + 审查 + CI 通过）

### 2.2 Fork 上游仓库 ✅

| 上游仓库 | SF 仓库 | 状态 |
|----------|---------|------|
| TurboWarp/scratch-vm | sf-vm | ✅ |
| TurboWarp/scratch-blocks | sf-blocks | ✅ |
| TurboWarp/scratch-render | sf-renderer | ✅ |
| TurboWarp/scratch-gui | sf-editor | ✅ |
| TurboWarp/extensions | sf-extensions | ✅ |
| TurboWarp/packager | sf-packager | ✅ |
| TurboWarp/desktop | sf-desktop | ✅ |
| TurboWarp/scratch-parser | sf-parser | ✅ |
| TurboWarp/scratch-storage | sf-storage | ✅ |
| TurboWarp/scratch-audio | sf-audio | ✅ |
| TurboWarp/scratch-paint | sf-paint | ✅ |
| TurboWarp/scratch-l10n | sf-l10n | ✅ |
| TurboWarp/cloud-server | sf-cloud-server | ✅ |

### 2.3 新建仓库 ✅

```bash
# 创建 Monorepo 核心仓库 (均已创建并推送到 GitHub)
git init sf-core       # ✅ 含完整 Cargo workspace + 6 crates
git init sf-tools      # ✅ 含 CLI 脚手架
git init sf-services   # ✅ 含 Cloudflare Workers + Hono 框架
git init sf-docs       # ✅ 含 VitePress 文档站
git init sf-runtime    # ✅ 含 sf CLI 命令行运行时
git init sf-aot-compiler  # ✅ AOT 编译器仓库（架构文档新增）
```

### 2.4 配置 Monorepo 根 ✅

在 sf-core/ 根目录创建 `Cargo.toml`:

```toml
[workspace]
members = [
    "sf-vm",
    "sf-blocks",
    "sf-renderer",
    "sf-parser",
    "sf-storage",
    "sf-audio",
]
resolver = "3"  # Cargo 2024 edition 使用 resolver 3
```

创建 `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
targets = ["wasm32-unknown-unknown"]
components = ["rustfmt", "clippy"]
```

> ✅ 已完成：还额外创建了 workspace.dependencies 统一依赖版本、.editorconfig、.github/CODEOWNERS、CI/CD GitHub Actions 流水线（check/fmt/clippy/test/wasm-build/security audit）、release.yml、dependabot.yml、所有仓库的分支保护规则。

---

## 3. Phase 2: 核心引擎 Rust 重写

### 3.1 sf-vm 数据模型与解析

#### 3.1.1 初始化 crate

```bash
cd sf-core
mkdir sf-vm && cd sf-vm
cargo init --lib --name sf-vm
```

#### 3.1.2 配置 Cargo.toml

```toml
[package]
name = "sf-vm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
zip = "2"
rusqlite = { version = "0.33", features = ["bundled"] }
base64 = "0.22"
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"
proptest = "1"
```

> **注意**：不再使用 `wee_alloc`。该项目已停止维护且存在内存泄漏问题。Rust 默认分配器配合 `wasm-opt -Os` 已提供更优的性能和更小的体积。

#### 3.1.3 实现数据模型

创建 `src/project/mod.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub meta: ProjectMeta,
    pub targets: Vec<Target>,
    pub extensions: Vec<String>,
    pub monitors: Vec<Monitor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub semver: String,
    pub vm: String,
    pub agent: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub created: String,
    pub modified: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub is_stage: bool,
    pub name: String,
    pub variables: HashMap<String, Variable>,
    pub lists: HashMap<String, List>,
    pub blocks: HashMap<String, Block>,
    pub broadcasts: HashMap<String, Broadcast>,
    pub comments: HashMap<String, Comment>,
    pub costumes: Vec<Costume>,
    pub sounds: Vec<Sound>,
    pub current_costume: usize,
    pub volume: f64,
    pub layer_order: usize,
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub direction: f64,
    pub draggable: bool,
    pub rotation_style: String,
}

// Block, Input, Field, Value, Costume, Sound, Variable, List, Broadcast, Comment, Monitor 等结构体...
// 完整实现见设计文档 04-editor.md 和 02-concepts.md
```

#### 3.1.4 实现 .sf 文件加载 (SQLite)

创建 `src/project/load_sf.rs`:

```rust
use rusqlite::Connection;

impl Project {
    pub fn load_sf(path: &str) -> Result<Self, VmError> {
        let conn = Connection::open(path)?;

        // 读取元信息
        let meta = conn.query_row("SELECT * FROM project_meta WHERE id = 1", [], |row| {
            Ok(ProjectMeta { /* ... */ })
        })?;

        // 读取 targets
        let mut stmt = conn.prepare("SELECT * FROM targets")?;
        let targets = stmt.query_map([], |row| {
            Ok(Target { /* ... */ })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(Project { meta, targets, /* ... */ })
    }
}
```

#### 3.1.5 实现 .sb3 文件加载 (ZIP)

```rust
impl Project {
    pub fn load_sb3(data: &[u8]) -> Result<Self, VmError> {
        let reader = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(reader)?;
        let project_json = archive.by_name("project.json")?;
        let project: Project = serde_json::from_reader(project_json)?;
        Ok(project)
    }
}
```

### 3.2 sf-vm 编译器

#### 3.2.1 创建编译器模块

创建 `src/compiler/mod.rs`:

```rust
pub mod js_generator;

use crate::project::Project;

pub fn compile(project: &Project) -> Result<String, CompileError> {
    let mut code = String::new();

    // 生成类声明
    code.push_str("class Project {\n");
    code.push_str("  constructor() {\n");
    code.push_str("    this.targets = {};\n");
    code.push_str("  }\n");

    // 为每个 target 生成方法
    for target in &project.targets {
        let target_code = compile_target(target)?;
        code.push_str(&target_code);
    }

    code.push_str("}\n");
    Ok(code)
}

fn compile_target(target: &Target) -> Result<String, CompileError> {
    // 遍历积木树，生成 JS 代码
    // ...
}
```

#### 3.2.2 实现 JS 代码生成器

创建 `src/compiler/js_generator.rs`:

```rust
use crate::ops::opcode::Opcode;

pub fn generate_block_js(opcode: &Opcode, args: &HashMap<String, String>) -> String {
    match opcode {
        Opcode::MotionMoveSteps => format!("await this.moveSteps({});", args["STEPS"]),
        Opcode::LooksSay => format!("await this.say(\"{}\");", args["MESSAGE"]),
        Opcode::ControlRepeat => {
            let count = &args["TIMES"];
            let body = &args["SUBSTACK"];
            format!("for (let i = 0; i < {}; i++) {{\n{}\n}}", count, body)
        }
        // ... 所有 125+ 操作码
        _ => "".to_string()
    }
}
```

### 3.3 sf-vm 执行器

#### 3.3.1 创建运行时状态

创建 `src/runtime/mod.rs`:

```rust
pub struct RuntimeState {
    pub targets: Vec<TargetState>,
    pub stage_index: usize,
    pub event_queue: VecDeque<RuntimeEvent>,
    pub clones: Vec<CloneData>,
    pub timer: f64,
    pub running: bool,
}

pub struct TargetState {
    pub id: String,
    pub name: String,
    pub variables: HashMap<String, Value>,
    pub lists: HashMap<String, Vec<Value>>,
    pub x: f64,
    pub y: f64,
    pub direction: f64,
    pub size: f64,
    pub visible: bool,
    pub current_costume: usize,
    pub volume: f64,
    pub pen_down: bool,
    pub pen_color: String,
    pub pen_size: f64,
}
```

#### 3.3.2 实现操作函数

创建 `src/ops/motion.rs`:

```rust
use crate::runtime::RuntimeState;

pub fn move_steps(state: &mut RuntimeState, target_idx: usize, steps: f64) {
    let target = &mut state.targets[target_idx];
    let angle = target.direction.to_radians();
    target.x += steps * angle.cos();
    target.y += steps * angle.sin();

    // 边缘反弹检查
    const BOUNCE_ANGLE_OFFSET: f64 = 180.0;
    if target.x > STAGE_MAX_WIDTH as f64 || target.x < STAGE_MIN_WIDTH as f64 {
        target.direction = BOUNCE_ANGLE_OFFSET - target.direction;
    }
    if target.y > STAGE_MAX_HEIGHT as f64 || target.y < STAGE_MIN_HEIGHT as f64 {
        target.direction = -target.direction;
    }
}
```

创建 `src/ops/mod.rs`，为每个积木类别创建子模块：`motion.rs`, `looks.rs`, `sound.rs`, `pen.rs`, `events.rs`, `control.rs`, `sensing.rs`, `operators.rs`, `variables.rs`。

### 3.4 sf-vm 扩展系统

创建 `src/extension/mod.rs`:

```rust
pub trait SfExtension: Send + Sync {
    fn info(&self) -> ExtensionInfo;
    fn blocks(&self) -> Vec<BlockDefinition>;
    fn execute(&self, opcode: &str, args: &[Value], state: &mut RuntimeState) -> Option<Value>;
    fn settings(&self) -> Vec<SettingDefinition> { vec![] }
    fn on_install(&self) {}
    fn on_uninstall(&self) {}
}

pub struct ExtensionManager {
    extensions: Vec<Box<dyn SfExtension>>,
}

impl ExtensionManager {
    pub fn register(&mut self, ext: Box<dyn SfExtension>) {
        self.extensions.push(ext);
    }

    pub fn execute(&self, opcode: &str, args: &[Value], state: &mut RuntimeState) -> Option<Value> {
        for ext in &self.extensions {
            if let Some(result) = ext.execute(opcode, args, state) {
                return Some(result);
            }
        }
        None
    }
}
```

### 3.5 sf-vm 设置引擎

创建 `src/settings/mod.rs`:

```rust
pub struct SettingsEngine {
    defaults: HashMap<String, Value>,
    user: HashMap<String, Value>,
    project: HashMap<String, Value>,
    session: HashMap<String, Value>,
    subscribers: HashMap<String, Vec<Box<dyn Fn(&Value)>>>,
}

impl SettingsEngine {
    pub fn get(&self, key: &str) -> Option<Value> {
        self.session.get(key)
            .or_else(|| self.project.get(key))
            .or_else(|| self.user.get(key))
            .or_else(|| self.defaults.get(key))
            .cloned()
    }

    pub fn set(&mut self, key: &str, value: Value, layer: SettingLayer) {
        match layer {
            SettingLayer::User => { self.user.insert(key.to_string(), value.clone()); }
            SettingLayer::Project => { self.project.insert(key.to_string(), value.clone()); }
            SettingLayer::Session => { self.session.insert(key.to_string(), value.clone()); }
        }
        // 通知订阅者
        if let Some(callbacks) = self.subscribers.get(key) {
            for cb in callbacks {
                cb(&value);
            }
        }
    }
}
```

### 3.6 sf-vm WASM 导出

在 `src/lib.rs` 中添加：

```rust
use wasm_bindgen::prelude::*;

// 初始化 panic hook，WASM panic 时输出完整调用栈到控制台
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn sf_vm_create() -> *mut RuntimeState {
    let state = Box::new(RuntimeState::new());
    Box::into_raw(state)
}

#[wasm_bindgen]
pub fn sf_vm_compile(state_ptr: *mut RuntimeState, project_json: &str) -> Result<String, JsValue> {
    // SAFETY: Reviewed — pointer comes from sf_vm_create which uses Box::into_raw
    let _state = unsafe { &mut *state_ptr };
    let project: Project = serde_json::from_str(project_json)?;
    compile(&project).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn sf_vm_execute(
    state_ptr: *mut RuntimeState,
    opcode: &str,
    args_json: &str
) -> Result<String, JsValue> {
    // SAFETY: Reviewed — pointer comes from sf_vm_create which uses Box::into_raw
    let state = unsafe { &mut *state_ptr };
    let args: HashMap<String, Value> = serde_json::from_str(args_json)?;
    // 执行积木逻辑...
    Ok(serde_json::to_string(&result)?)
}

#[wasm_bindgen]
pub fn sf_vm_destroy(state_ptr: *mut RuntimeState) {
    // SAFETY: Reviewed — pointer was created by sf_vm_create via Box::into_raw
    // Caller must not use state_ptr after calling destroy
    unsafe { drop(Box::from_raw(state_ptr)); }
}
```

### 3.7 sf-blocks 积木编辑器

#### 3.7.1 初始化 crate

```bash
cd sf-core
mkdir sf-blocks && cd sf-blocks
cargo init --lib --name sf-blocks
```

#### 3.7.2 配置 Cargo.toml

```toml
[package]
name = "sf-blocks"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["CanvasRenderingContext2d", "Element", "Event", "MouseEvent"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
lyon_geom = "1"
```

#### 3.7.3 实现积木布局引擎

创建 `src/layout.rs`:

```rust
pub struct BlockLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub notch_x: f64,
    pub bump_x: f64,
}

pub fn compute_layout(opcode: &str, fields: &HashMap<String, String>) -> BlockLayout {
    let text = fields.get("TEXT").cloned().unwrap_or_default();
    let text_width = text.len() as f64 * 8.0; // 估算文本宽度
    let width = (text_width + 32.0).max(64.0); // 最小宽度 64px

    BlockLayout {
        width,
        height: 32.0,
        notch_x: width / 2.0 - 4.0,
        bump_x: width / 2.0 - 4.0,
        x: 0.0,
        y: 0.0,
    }
}
```

#### 3.7.4 实现 Canvas 渲染器

创建 `src/renderer.rs`:

```rust
use web_sys::CanvasRenderingContext2d;

const CATEGORY_COLORS: &[(&str, &str)] = &[
    ("motion", "#4C97FF"),
    ("looks", "#9966FF"),
    ("sound", "#CF63CF"),
    ("events", "#DE9E2C"),
    ("control", "#FFAB19"),
    ("sensing", "#5CB1D6"),
    ("operators", "#59C059"),
    ("variables", "#FF8C1A"),
    ("myblocks", "#FF6680"),
];

pub fn draw_block(ctx: &CanvasRenderingContext2d, layout: &BlockLayout, category: &str, text: &str) {
    let color = CATEGORY_COLORS.iter()
        .find(|(cat, _)| *cat == category)
        .map(|(_, c)| *c)
        .unwrap_or("#CCCCCC");

    // 绘制积木背景
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    // 圆角矩形
    ctx.rect(layout.x, layout.y, layout.width, layout.height);
    ctx.fill();

    // 绘制文字
    ctx.set_fill_style_str("#FFFFFF");
    ctx.set_font("13px sans-serif");
    ctx.fill_text(text, layout.x + 8.0, layout.y + 22.0)?;
}
```

#### 3.7.5 实现拖拽与吸附

创建 `src/drag.rs`:

```rust
const SNAP_DISTANCE: f64 = 8.0;

pub struct DragState {
    pub dragging: bool,
    pub block_id: Option<String>,
    pub offset_x: f64,
    pub offset_y: f64,
}

pub fn check_snap(mouse_x: f64, mouse_y: f64, target_x: f64, target_y: f64) -> bool {
    let dx = mouse_x - target_x;
    let dy = mouse_y - target_y;
    (dx * dx + dy * dy).sqrt() < SNAP_DISTANCE
}
```

#### 3.7.6 实现动态模糊拖尾

创建 `src/trail.rs`:

```rust
const TRAIL_FRAMES: usize = 6;
const TRAIL_ALPHA_START: f64 = 0.8;
const TRAIL_ALPHA_END: f64 = 0.1;

pub struct TrailRenderer {
    positions: VecDeque<(f64, f64)>,
}

impl TrailRenderer {
    pub fn record(&mut self, x: f64, y: f64) {
        self.positions.push_back((x, y));
        if self.positions.len() > TRAIL_FRAMES {
            self.positions.pop_front();
        }
    }

    pub fn render(&self, ctx: &CanvasRenderingContext2d, block_width: f64, block_height: f64) {
        for (i, (x, y)) in self.positions.iter().enumerate() {
            let alpha = TRAIL_ALPHA_START - (i as f64) * (TRAIL_ALPHA_START - TRAIL_ALPHA_END) / TRAIL_FRAMES as f64;
            ctx.set_global_alpha(alpha);
            ctx.set_stroke_style_str("#2563EB");
            ctx.stroke_rect(*x, *y, block_width, block_height);
        }
        ctx.set_global_alpha(1.0);
    }
}
```

### 3.8 sf-renderer 舞台渲染器

#### 3.8.1 初始化 crate

```bash
cd sf-core
mkdir sf-renderer && cd sf-renderer
cargo init --lib --name sf-renderer
```

#### 3.8.2 配置 Cargo.toml

```toml
[package]
name = "sf-renderer"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["WebGl2RenderingContext", "CanvasRenderingContext2d"] }
lyon = "1"
resvg = "0.44"
```

#### 3.8.3 实现 WebGL2 渲染管线

创建 `src/webgl.rs`:

```rust
use web_sys::WebGl2RenderingContext;

pub fn init_webgl(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
    canvas.get_context("webgl2")?.map(|ctx| ctx.dyn_into::<WebGl2RenderingContext>().unwrap())
}

pub fn compile_shader(gl: &WebGl2RenderingContext, source: &str, shader_type: u32) -> Result<WebGlShader, String> {
    let shader = gl.create_shader(shader_type).ok_or("Cannot create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_default())
    }
}
```

#### 3.8.4 实现 SVG 解析

创建 `src/svg.rs`:

```rust
use resvg::usvg;

pub fn parse_svg(data: &[u8]) -> Result<usvg::Tree, Box<dyn std::error::Error>> {
    let tree = usvg::Tree::from_data(data, &usvg::Options::default())?;
    Ok(tree)
}

pub fn triangulate(tree: &usvg::Tree) -> Vec<Vertex> {
    // 使用 lyon 将 SVG 路径三角化
    // ...
    vec![]
}
```

### 3.9 sf-parser 解析器

#### 3.9.1 初始化

```bash
cd sf-core
mkdir sf-parser && cd sf-parser
cargo init --lib --name sf-parser
```

#### 3.9.2 配置

```toml
[package]
name = "sf-parser"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
zip = "2"
rusqlite = { version = "0.33", features = ["bundled"] }
thiserror = "2"
```

#### 3.9.3 实现

创建 `src/lib.rs`:

```rust
pub fn parse_sb3(data: &[u8]) -> Result<serde_json::Value, ParseError> {
    let reader = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader)?;
    let project = archive.by_name("project.json")?;
    let value = serde_json::from_reader(project)?;
    Ok(value)
}

pub fn parse_sf(path: &str) -> Result<serde_json::Value, ParseError> {
    let conn = rusqlite::Connection::open(path)?;
    // 读取数据并转换为 JSON
    // ...
    Ok(serde_json::Value::Null)
}

pub fn parse_sfl(source: &str) -> Result<serde_json::Value, ParseError> {
    // .sfl 文本解析为 JSON AST
    // ...
    Ok(serde_json::Value::Null)
}
```

---

## 4. Phase 3: 编辑器 UI 框架

### 4.1 初始化 sf-editor

```bash
cd sf-editor
pnpm create next-app@latest . --typescript --turbopack --app
```

### 4.2 安装依赖

```bash
pnpm add zustand @phosphor-icons/react react-aria-components @tanstack/react-table next-intl
pnpm add -D vitest @testing-library/react @playwright/test @axe-core/playwright
```

### 4.3 配置构建

`next.config.ts`:

```typescript
const nextConfig = {
  output: 'export',
  turbopack: {
    rules: { '*.wasm': { loaders: ['wasm'] } }
  }
};
```

### 4.4 实现设计令牌

创建 `src/design/tokens/`，按设计文档实现颜色、字体、间距、圆角、阴影、动画令牌。

### 4.5 实现 UI 组件库

在 `src/components/` 下实现按钮、输入框、下拉菜单、弹窗、树形视图、选项卡、工具栏、面板、状态栏。

### 4.6 实现编辑器布局

创建 `src/app/layout.tsx` 和 `src/app/page.tsx`，组装标题栏、菜单、工具栏、工具箱、积木画布、舞台、右侧面板。

### 4.7 加载 WASM 模块

创建 `src/lib/wasm.ts`:

```typescript
let vmModule: any = null;

export async function loadWasm() {
  if (!vmModule) {
    vmModule = await import('../../wasm/sf-vm/sf_vm.js');
    await vmModule.default();
  }
  return vmModule;
}
```

### 4.8 实现状态管理

使用 Zustand 创建 `useProjectStore`, `useEditorStore`, `useSettingsStore`。

---

## 5. Phase 4: 多人协作系统

### 5.1 协作服务器搭建

- 使用 Rust + WebSocket 构建信令服务器
- 实现房间管理、消息广播

### 5.2 实现 OT 算法

- 积木树操作的变换函数
- 变量操作的 LWW 策略

### 5.3 实现用户存在感

- 光标位置同步
- 编辑状态广播

### 5.4 实现用户系统

- 注册/登录 API
- OAuth 集成

### 5.5 实现离线同步

- IndexedDB 操作日志
- 恢复连接同步流程

---

## 6. Phase 5: 运行时与打包器

### 6.1 SF Runtime CLI

```bash
cd sf-runtime
cargo init --name sf
```

使用 clap 构建 CLI:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sf", version, about = "Sailfish Studio Runtime")
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行 .sfl 或 .sfp 文件
    Run {
        #[arg(value_name = "FILE")]
        file: String,
        /// 有头模式（图形界面）
        #[arg(long)]
        headed: bool,
    },
    /// 打包为 .sfp
    Pack {
        #[arg(value_name = "FILE")]
        file: String,
    },
    /// 创建新项目
    New {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// 检查项目语法
    Check {
        #[arg(value_name = "FILE")]
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { file, headed } => { /* ... */ }
        Commands::Pack { file } => { /* ... */ }
        Commands::New { name } => { /* ... */ }
        Commands::Check { file } => { /* ... */ }
    }
}
```

### 6.2 sf-packager 增强

- 新增 SWF、MP4、GIF、APK 导出模块

### 6.3 AOT 编译器

- 后端使用 LLVM (cranelift 或 llvm-sys)
- 实现 .sf/.sfl → 原生可执行文件

---

## 7. Phase 6: 桌面端与移动端

### 7.1 Tauri 桌面端

```bash
cd sf-desktop
pnpm create tauri-app@latest .
```

- 配置 `tauri.conf.json`
- 实现 Rust 侧命令
- 打包分发

### 7.2 Android 平板端

- 独立 UI 适配
- 触控优化
- APK 打包

---

## 8. Phase 7: 扩展市场与生态

### 8.1 扩展市场网站

- 搜索、分类、详情页、一键安装

### 8.2 首批高级扩展开发

- 21 个扩展，使用 Rust/C++/TS

---

## 9. Phase 8: 测试与发布

### 9.1 单元测试

```bash
cargo nextest run     # Rust 单元测试
cargo tarpaulin       # Rust 覆盖率
pnpm test             # TypeScript 单元测试
cargo fuzz run fuzz_target  # 模糊测试
```

### 9.2 E2E 测试

```bash
pnpm e2e              # Playwright
```

### 9.3 发布

详细发布流程见 `docs/development/08-release-process.md`。

1. 在 develop 分支确认所有功能冻结
2. 更新 CHANGELOG.md
3. 提交 PR 合并到 main
4. 打 Tag: `git tag v1.0.0-beta`
5. CI 自动构建并发布:
   - Web 端部署到 Cloudflare Pages
   - 桌面端发布到 GitHub Releases
   - 运行时 CLI 发布到 npm / crates.io
6. 回滚策略:
   - Cloudflare Pages: 在 Dashboard 中选择上一版本回滚
   - 桌面端: GitHub Releases 保留旧版本下载链接
