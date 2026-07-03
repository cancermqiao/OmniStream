pub const CSS: &str = r#"
* { box-sizing: border-box; }
html, body {
  width: 100%;
  max-width: 100%;
  overflow-x: hidden;
}
:root {
  color-scheme: light;
  --ink: #0c1222;
  --muted: #6a7283;
  --soft: #9aa4b5;
  --line: rgba(15, 23, 42, 0.08);
  --panel: rgba(255, 255, 255, 0.70);
  --panel-strong: rgba(255, 255, 255, 0.88);
  --accent: #1268ff;
  --accent-strong: #0b55dc;
  --success: #22c55e;
  --warning: #f59e0b;
  --danger: #ff3b30;
  --radius-lg: 24px;
  --radius-md: 16px;
  --sidebar-width: 178px;
  --sidebar-collapsed-width: 60px;
  --shadow-soft: 0 22px 68px rgba(15, 23, 42, 0.11);
}

body {
  margin: 0;
  background:
    radial-gradient(circle at 8% 0%, rgba(42, 99, 255, 0.15), transparent 26%),
    radial-gradient(circle at 94% 2%, rgba(91, 210, 186, 0.15), transparent 24%),
    linear-gradient(135deg, #f5f8ff 0%, #eef3f8 45%, #f8fbff 100%);
  color: var(--ink);
  font-family: Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', Arial, sans-serif;
  font-size: 12px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
}

.app-shell {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}
.layout {
  display: grid;
  grid-template-columns: var(--sidebar-width) minmax(0, calc(100vw - var(--sidebar-width)));
  height: 100vh;
  width: 100vw;
  max-width: 100vw;
  transition: grid-template-columns 0.22s ease;
  overflow: hidden;
}

.layout-collapsed {
  grid-template-columns: var(--sidebar-collapsed-width) minmax(0, calc(100vw - var(--sidebar-collapsed-width)));
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  background:
    radial-gradient(circle at 16% 0%, rgba(40, 99, 255, 0.28), transparent 25%),
    radial-gradient(circle at 90% 20%, rgba(70, 219, 190, 0.14), transparent 24%),
    linear-gradient(180deg, #090d19 0%, #0a1020 56%, #050915 100%);
  color: #e5eefc;
  padding: 20px 10px;
  border-right: 1px solid rgba(255,255,255,0.10);
  box-shadow: 18px 0 70px rgba(7, 14, 29, 0.22);
  overflow: hidden;
  transition: padding 0.22s ease;
}

.sidebar-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 24px;
}

.sidebar-collapsed {
  padding: 18px 8px;
}

.brand {
  font-weight: 760;
  font-size: 17px;
  letter-spacing: -0.045em;
  color: #f8fafc;
  white-space: nowrap;
}

.sidebar-collapsed .brand {
  width: 30px;
  overflow: hidden;
  font-size: 19px;
}

.subtitle {
  margin: 6px 0 20px;
  color: rgba(226, 232, 240, 0.58);
  font-size: 10px;
  line-height: 1.45;
}

.sidebar-toggle {
  width: 28px;
  height: 28px;
  flex: 0 0 auto;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(248, 250, 252, 0.88);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  transition: background 0.16s ease, transform 0.16s ease;
}

.sidebar-toggle:hover {
  background: rgba(255, 255, 255, 0.16);
  transform: scale(1.04);
}

.tab-item {
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  color: rgba(226, 232, 240, 0.76);
  border-radius: 14px;
  padding: 10px 10px;
  font-weight: 680;
  font-size: 11px;
  cursor: pointer;
  margin-bottom: 7px;
  display: flex;
  align-items: center;
  gap: 9px;
  transition: background 0.18s ease, color 0.18s ease, transform 0.18s ease, box-shadow 0.18s ease;
}

.tab-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
  transform: translateX(3px);
}
.tab-item-active {
  background: rgba(255, 255, 255, 0.14);
  color: white;
  box-shadow: inset 0 0 0 1px rgba(255,255,255,0.14), 0 16px 34px rgba(0, 0, 0, 0.18);
  backdrop-filter: blur(16px);
}

.tab-icon {
  width: 24px;
  height: 24px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 9px;
  background: rgba(255, 255, 255, 0.08);
  font-size: 13px;
  line-height: 1;
}

.tab-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-collapsed .tab-item {
  justify-content: center;
  padding: 12px 0;
}

.sidebar-collapsed .tab-item:hover {
  transform: translateY(-1px);
}

.content {
  min-width: 0;
  width: auto;
  max-width: 100%;
  height: 100vh;
  padding: 22px clamp(12px, 1.4vw, 20px) 32px;
  overflow-y: auto;
  overflow-x: hidden;
}
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11px; }

@media (max-width: 980px) {
  .layout,
  .layout-collapsed { grid-template-columns: minmax(0, 1fr); }
  .sidebar {
    position: sticky;
    top: 0;
    height: auto;
    z-index: 10;
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    padding: 18px;
  }
  .sidebar-top { width: 100%; margin-bottom: 4px; }
  .brand { width: 100%; }
  .subtitle { width: 100%; margin-top: -4px; }
  .tab-item { width: auto; margin-bottom: 0; }
  .content { padding: 18px; }
}
"#;
