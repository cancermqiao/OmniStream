pub const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Manrope:wght@400;500;600;700;800&display=swap');

* { box-sizing: border-box; }
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
  font-family: 'Manrope', -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', sans-serif;
  -webkit-font-smoothing: antialiased;
}

.app-shell { min-height: 100vh; }
.layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  min-height: 100vh;
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  background:
    radial-gradient(circle at 12% 8%, rgba(255, 255, 255, 0.13), transparent 24%),
    linear-gradient(180deg, #050816 0%, #0b1220 52%, #0f172a 100%);
  color: #e5eefc;
  padding: 28px 18px;
  border-right: 1px solid rgba(255,255,255,0.10);
  box-shadow: 28px 0 80px rgba(15, 23, 42, 0.16);
}

.brand {
  font-weight: 800;
  font-size: 28px;
  letter-spacing: -0.055em;
  color: #f8fafc;
}

.subtitle {
  margin: 7px 0 28px;
  color: rgba(226, 232, 240, 0.66);
  font-size: 13px;
  line-height: 1.5;
}

.tab-item {
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  color: rgba(226, 232, 240, 0.76);
  border-radius: 18px;
  padding: 14px 16px;
  font-weight: 700;
  font-size: 14px;
  cursor: pointer;
  margin-bottom: 8px;
  transition: background 0.18s ease, color 0.18s ease, transform 0.18s ease;
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

.content {
  width: min(1640px, 100%);
  padding: 36px 32px 48px;
}
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; }

@media (max-width: 980px) {
  .layout { grid-template-columns: 1fr; }
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
  .brand { width: 100%; }
  .subtitle { width: 100%; margin-top: -4px; }
  .tab-item { width: auto; margin-bottom: 0; }
  .content { padding: 18px; }
}
"#;
