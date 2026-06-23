pub const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Manrope:wght@400;500;600;700;800&display=swap');

* { box-sizing: border-box; }
body {
  margin: 0;
  background:
    radial-gradient(circle at top left, rgba(14, 165, 163, 0.13), transparent 30%),
    linear-gradient(180deg, #f8fbff 0%, #f1f5f9 100%);
  color: #0f172a;
  font-family: 'Manrope', sans-serif;
}

.app-shell { min-height: 100vh; }
.layout {
  display: grid;
  grid-template-columns: 250px 1fr;
  min-height: 100vh;
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  background: linear-gradient(180deg, #0f172a 0%, #111827 100%);
  color: #dbeafe;
  padding: 24px 14px;
  border-right: 1px solid rgba(255,255,255,0.08);
  box-shadow: 16px 0 40px rgba(15, 23, 42, 0.08);
}

.brand {
  font-weight: 800;
  font-size: 24px;
  color: #f8fafc;
}

.subtitle {
  margin: 6px 0 18px;
  color: #94a3b8;
  font-size: 12px;
}

.tab-item {
  width: 100%;
  text-align: left;
  border: 0;
  background: transparent;
  color: #cbd5e1;
  border-radius: 10px;
  padding: 11px 12px;
  font-weight: 600;
  cursor: pointer;
  margin-bottom: 6px;
  transition: background 0.16s ease, color 0.16s ease, transform 0.16s ease;
}

.tab-item:hover {
  background: rgba(148, 163, 184, 0.18);
  transform: translateX(2px);
}
.tab-item-active {
  background: linear-gradient(135deg, #0f766e, #14b8a6);
  color: white;
  box-shadow: 0 10px 24px rgba(20, 184, 166, 0.24);
}

.content {
  width: min(1500px, 100%);
  padding: 28px;
}
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11px; }

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
  }
  .brand { width: 100%; }
  .subtitle { width: 100%; margin-top: -4px; }
  .tab-item { width: auto; margin-bottom: 0; }
  .content { padding: 14px; }
}
"#;
