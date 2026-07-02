pub const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Manrope:wght@400;500;600;700;800&display=swap');

* { box-sizing: border-box; }
html, body {
  width: 100%;
  max-width: 100%;
  overflow-x: hidden;
}
:root {
  color-scheme: light;
  --ink: #111827;
  --muted: #6b7280;
  --soft: #9ca3af;
  --line: rgba(17, 24, 39, 0.08);
  --panel: rgba(255, 255, 255, 0.74);
  --panel-strong: rgba(255, 255, 255, 0.92);
  --accent: #007aff;
  --accent-strong: #0066d6;
  --success: #22c55e;
  --warning: #f59e0b;
  --danger: #ff3b30;
  --radius-lg: 28px;
  --radius-md: 18px;
  --shadow-soft: 0 24px 80px rgba(15, 23, 42, 0.10);
}

body {
  margin: 0;
  background:
    radial-gradient(circle at 12% 8%, rgba(0, 122, 255, 0.13), transparent 28%),
    radial-gradient(circle at 88% 0%, rgba(52, 199, 89, 0.10), transparent 26%),
    linear-gradient(180deg, #f8fafc 0%, #eef3f8 48%, #f7f8fb 100%);
  color: var(--ink);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', 'Inter', 'Manrope', sans-serif;
  font-size: 14px;
  line-height: 1.45;
  -webkit-font-smoothing: antialiased;
  text-rendering: geometricPrecision;
}

.app-shell {
  min-height: 100vh;
  width: 100%;
  max-width: 100vw;
  overflow-x: hidden;
}
.layout {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr);
  min-height: 100vh;
  width: 100%;
  max-width: 100vw;
  transition: grid-template-columns 0.22s ease;
}

.layout-collapsed {
  grid-template-columns: 78px minmax(0, 1fr);
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  background:
    radial-gradient(circle at 12% 8%, rgba(255, 255, 255, 0.13), transparent 24%),
    linear-gradient(180deg, #050816 0%, #0b1220 52%, #0f172a 100%);
  color: #e5eefc;
  padding: 24px 16px;
  border-right: 1px solid rgba(255,255,255,0.10);
  box-shadow: 28px 0 80px rgba(15, 23, 42, 0.16);
  overflow: hidden;
  transition: padding 0.22s ease;
}

.sidebar-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 24px;
}

.sidebar-collapsed {
  padding: 22px 12px;
}

.brand {
  font-weight: 740;
  font-size: 24px;
  letter-spacing: -0.045em;
  color: #f8fafc;
  white-space: nowrap;
}

.sidebar-collapsed .brand {
  width: 32px;
  overflow: hidden;
  font-size: 21px;
}

.subtitle {
  margin: 7px 0 28px;
  color: rgba(226, 232, 240, 0.66);
  font-size: 13px;
  line-height: 1.5;
}

.sidebar-toggle {
  width: 32px;
  height: 32px;
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
  border-radius: 18px;
  padding: 12px 14px;
  font-weight: 650;
  font-size: 13px;
  cursor: pointer;
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 11px;
  transition: background 0.18s ease, color 0.18s ease, transform 0.18s ease, box-shadow 0.18s ease;
}

.tab-item:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
  transform: translateX(3px);
}
.tab-item-active {
  background: rgba(255, 255, 255, 0.15);
  color: white;
  box-shadow: inset 0 0 0 1px rgba(255,255,255,0.14), 0 18px 36px rgba(0, 0, 0, 0.18);
  backdrop-filter: blur(16px);
}

.tab-icon {
  width: 28px;
  height: 28px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.08);
  font-size: 15px;
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
  width: 100%;
  max-width: 100%;
  padding: 32px clamp(18px, 2.4vw, 32px) 44px;
  overflow-x: hidden;
}
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; }

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
