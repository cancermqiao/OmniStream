pub const CSS: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Manrope:wght@400;500;600;700;800&display=swap');

* { box-sizing: border-box; }
body {
  margin: 0;
  background: #f6f8fb;
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
  background: #0f172a;
  color: #dbeafe;
  padding: 24px 14px;
  border-right: 1px solid rgba(255,255,255,0.08);
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
}

.tab-item:hover { background: rgba(148, 163, 184, 0.18); }
.tab-item-active {
  background: #0ea5a3;
  color: white;
}

.content { padding: 28px; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11px; }

@media (max-width: 980px) {
  .layout { grid-template-columns: 1fr; }
  .sidebar {
    position: sticky;
    top: 0;
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
