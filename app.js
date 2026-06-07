/* ============================================================
   Sailfish Studio - 文档浏览器核心脚本
   功能: SPA 导航, Markdown 渲染, 语法高亮, 响应式侧栏
   ============================================================ */

// ── 文档索引 ──
const SECTIONS = [
  {
    title: '需求规格',
    base: 'docs/requirements/',
    docs: [
      { file: 'README.md',         label: 'README',         num: '' },
      { file: '00-constants.md',   label: '文档约定与常量', num: '00' },
      { file: '01-overview.md',     label: '项目概述',       num: '01' },
      { file: '02-concepts.md',     label: '核心概念',       num: '02' },
      { file: '03-architecture.md', label: '技术架构',      num: '03' },
      { file: '04-editor.md',       label: '积木编辑器',     num: '04' },
      { file: '05-collaboration.md',label: '多人协作',      num: '05' },
      { file: '06-ui.md',          label: 'UI 与设计语言',  num: '06' },
      { file: '07-extensions.md',   label: '扩展与插件',     num: '07' },
      { file: '08-runtime.md',      label: '运行时与打包',   num: '08' },
      { file: '09-platform.md',     label: '跨平台',         num: '09' },
      { file: '10-debug.md',        label: '调试与日志',     num: '10' },
      { file: '11-nfr.md',          label: '非功能性需求',   num: '11' },
      { file: '12-roadmap.md',      label: '路线图',         num: '12' },
      { file: '13-appendices.md',   label: '附录',           num: '13' },
      { file: '14-changelog.md',    label: '变更历史',       num: '14' },
    ]
  },
  {
    title: '架构设计',
    base: 'docs/design/architecture/',
    docs: [
      { file: '01-process-model.md',   label: '进程模型',     num: 'A1' },
      { file: '02-ipc-protocol.md',    label: 'IPC 协议',    num: 'A2' },
      { file: '03-data-flow.md',       label: '数据流',       num: 'A3' },
      { file: '04-deployment.md',      label: '部署拓扑',     num: 'A4' },
    ]
  },
  {
    title: 'UI 设计 - 基础',
    base: 'docs/design/ui/',
    docs: [
      { file: '01-design-principles.md',  label: '设计原则',     num: 'U1' },
      { file: '02-color-system.md',       label: '色彩体系',     num: 'U2' },
      { file: '03-typography.md',         label: '字体排版',     num: 'U3' },
      { file: '04-spacing-grid.md',       label: '间距网格',     num: 'U4' },
      { file: '05-iconography.md',        label: '图标系统',     num: 'U5' },
      { file: '06-motion.md',             label: '动画规范',     num: 'U6' },
    ]
  },
  {
    title: 'UI 设计 - 组件',
    base: 'docs/design/ui/07-components/',
    docs: [
      { file: 'button.md',        label: '按钮',       num: 'C1' },
      { file: 'input.md',         label: '输入框',     num: 'C2' },
      { file: 'dialog.md',        label: '对话框',     num: 'C3' },
      { file: 'dropdown.md',      label: '下拉菜单',   num: 'C4' },
      { file: 'tab-bar.md',       label: '选项卡',     num: 'C5' },
      { file: 'toolbar.md',       label: '工具栏',     num: 'C6' },
      { file: 'panel.md',         label: '面板',       num: 'C7' },
      { file: 'tree-view.md',     label: '树形视图',   num: 'C8' },
      { file: 'block-canvas.md',  label: '积木画布',   num: 'C9' },
      { file: 'stage-canvas.md',  label: '舞台画布',   num: 'CA' },
      { file: 'status-bar.md',    label: '状态栏',     num: 'CB' },
    ]
  },
  {
    title: 'UI 设计 - 布局与适配',
    base: 'docs/design/ui/',
    docs: [
      { file: '08-layout.md',          label: '编辑器布局',  num: 'U7' },
      { file: '09-dark-mode.md',       label: '深色模式',    num: 'U8' },
      { file: '10-accessibility.md',   label: '无障碍',      num: 'U9' },
      { file: '11-mobile-tablet.md',   label: '平板适配',    num: 'UA' },
    ]
  },
  {
    title: '语言设计',
    base: 'docs/design/language/',
    docs: [
      { file: '01-lexical-grammar.md', label: '词法规则',    num: 'L1' },
      { file: '02-syntax.md',          label: '语法规范',    num: 'L2' },
      { file: '03-type-system.md',     label: '类型系统',    num: 'L3' },
      { file: '04-block-mapping.md',   label: '积木映射',    num: 'L4' },
      { file: '05-standard-library.md',label: '标准库',      num: 'L5' },
      { file: '06-vscode-extension.md',label: 'VS Code 扩展', num: 'L6' },
    ]
  },
  {
    title: '扩展系统',
    base: 'docs/design/extensions/',
    docs: [
      { file: '01-extension-api.md',      label: '扩展 API',        num: 'E1' },
      { file: '02-extension-lifecycle.md', label: '扩展生命周期',   num: 'E2' },
      { file: '03-plugin-api.md',          label: '插件 API',       num: 'E3' },
      { file: '04-sandbox-model.md',       label: '沙箱模型',       num: 'E4' },
      { file: '05-marketplace.md',         label: '扩展市场',       num: 'E5' },
    ]
  },
  {
    title: '协作设计',
    base: 'docs/design/collaboration/',
    docs: [
      { file: '01-protocol.md',             label: '协作协议',       num: 'C1' },
      { file: '02-room-server.md',          label: '房间服务器',     num: 'C2' },
      { file: '03-presence.md',             label: '用户存在感',     num: 'C3' },
      { file: '04-conflict-resolution.md',  label: '冲突解决',       num: 'C4' },
      { file: '05-offline-sync.md',         label: '离线同步',       num: 'C5' },
    ]
  },
  {
    title: '安全',
    base: 'docs/design/security/',
    docs: [
      { file: '01-threat-model.md',      label: '威胁模型',     num: 'S1' },
      { file: '02-sandbox-boundaries.md',label: '沙箱边界',     num: 'S2' },
      { file: '03-permission-model.md',  label: '权限模型',     num: 'S3' },
      { file: '04-csp-policy.md',        label: 'CSP 策略',     num: 'S4' },
      { file: '05-code-signing.md',      label: '代码签名',     num: 'S5' },
    ]
  },
  {
    title: '测试',
    base: 'docs/design/testing/',
    docs: [
      { file: '01-test-pyramid.md',          label: '测试金字塔',   num: 'T1' },
      { file: '02-performance-bench.md',     label: '性能基准',     num: 'T2' },
      { file: '03-compatibility-matrix.md',  label: '兼容性矩阵',   num: 'T3' },
      { file: '04-fuzzing-strategy.md',      label: '模糊测试',     num: 'T4' },
    ]
  },
];

// ── DOM 引用 ──
const contentEl = document.getElementById('content');
const navEl = document.getElementById('nav');
const breadcrumbEl = document.getElementById('breadcrumb');

// ── 构建全局文档索引 ──
let globalIdx = 0;
const docIndex = [];
SECTIONS.forEach(section => {
  section.docs.forEach(doc => {
    docIndex.push({ base: section.base, file: doc.file, label: doc.label, num: doc.num });
  });
});

// ── 构建侧栏导航 ──
function buildNav() {
  let idx = 0;
  SECTIONS.forEach(section => {
    const heading = document.createElement('div');
    heading.className = 'nav-heading';
    heading.textContent = section.title;
    navEl.appendChild(heading);
    section.docs.forEach(doc => {
      const a = document.createElement('a');
      a.className = 'nav-item';
      a.dataset.idx = idx;
      a.href = '#' + section.base + doc.file.replace('.md', '');
      const numSpan = doc.num ? `<span class="num">${doc.num}</span>` : '';
      a.innerHTML = `${numSpan}${doc.label}`;
      a.addEventListener('click', e => {
        e.preventDefault();
        loadDoc(idx);
        closeSidebar();
      });
      navEl.appendChild(a);
      idx++;
    });
  });
}

// ── 配置 marked ──
marked.setOptions({
  gfm: true,
  breaks: false,
  highlight: (code, lang) => {
    if (window.hljs && lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value;
      } catch (_) {}
    }
    return code;
  }
});

// ── 加载文档 ──
let currentIdx = -1;

async function loadDoc(idx) {
  if (idx === currentIdx) return;
  currentIdx = idx;
  const doc = docIndex[idx];
  if (!doc) return;

  // 更新导航高亮
  document.querySelectorAll('.nav-item').forEach(el => {
    el.classList.toggle('active', parseInt(el.dataset.idx) === idx);
  });

  // 面包屑
  const section = SECTIONS.find(s => s.docs.some(d => d.file === doc.file && s.base === doc.base));
  breadcrumbEl.innerHTML = `${section ? section.title : 'Sailfish Studio'} / <span>${doc.label}</span>`;

  // 加载状态
  contentEl.innerHTML = '<div class="loading"><div class="spinner"></div>加载文档中…</div>';

  try {
    const res = await fetch(doc.base + doc.file);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const md = await res.text();
    contentEl.innerHTML = '<div class="markdown-body">' + marked.parse(md) + '</div>';

    // 拦截内部链接
    contentEl.querySelectorAll('a[href^="./"]').forEach(a => {
      const href = a.getAttribute('href');
      const targetFile = href.replace('./', '');
      const targetIdx = docIndex.findIndex(d => {
        const fullPath = d.base + d.file;
        return fullPath === doc.base + targetFile || fullPath.endsWith('/' + targetFile);
      });
      if (targetIdx !== -1) {
        a.addEventListener('click', e => {
          e.preventDefault();
          loadDoc(targetIdx);
          contentEl.scrollTop = 0;
        });
      }
    });

    contentEl.scrollTop = 0;
  } catch (err) {
    contentEl.innerHTML = `<div class="error-state"><div class="icon">⚠</div><div class="msg">文档加载失败: ${err.message}</div></div>`;
  }
}

// ── 侧栏切换（移动端） ──
window.toggleSidebar = function() {
  document.getElementById('sidebar').classList.toggle('open');
  document.getElementById('overlay').classList.toggle('show');
};
window.closeSidebar = function() {
  document.getElementById('sidebar').classList.remove('open');
  document.getElementById('overlay').classList.remove('show');
};

// ── 初始化 ──
function init() {
  buildNav();
  const hash = location.hash.replace('#', '');
  const idx = docIndex.findIndex(d =>
    (d.base + d.file.replace('.md', '')).endsWith(hash) ||
    d.file.replace('.md', '') === hash
  );
  loadDoc(idx !== -1 ? idx : 0);
}
window.addEventListener('hashchange', init);
init();
