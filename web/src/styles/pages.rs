pub const CSS: &str = r#"
.page { display: flex; flex-direction: column; gap: 16px; }

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 16px;
  padding: 0 0 6px;
  min-width: 0;
}

.page-header h1 {
  margin: 0;
  font-size: clamp(26px, 2.2vw, 36px);
  line-height: 1.08;
  letter-spacing: -0.035em;
  font-weight: 700;
}

.page-header p {
  margin: 8px 0 0;
  color: var(--muted);
  font-size: 13px;
  line-height: 1.55;
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}

.stat-card {
  background: var(--panel);
  border: 1px solid rgba(255, 255, 255, 0.72);
  border-radius: 22px;
  padding: 15px 16px;
  box-shadow: 0 14px 42px rgba(15, 23, 42, 0.06);
  backdrop-filter: blur(22px);
  transition: transform 0.18s ease, box-shadow 0.18s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-soft);
}

.stat-label {
  margin: 0;
  color: var(--muted);
  font-size: 12px;
  font-weight: 650;
  letter-spacing: 0.01em;
}

.stat-value {
  margin: 7px 0 0;
  color: var(--ink);
  font-size: clamp(22px, 2vw, 30px);
  font-weight: 700;
  letter-spacing: -0.035em;
}

.stat-hint {
  margin: 5px 0 0;
  color: var(--soft);
  font-size: 11px;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  min-width: 0;
}

.toolbar + .toolbar {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--line);
}

.toolbar .input {
  min-width: min(260px, 100%);
  max-width: min(390px, 100%);
  flex: 1 1 260px;
}

.toolbar-label {
  font-size: 12px;
  color: var(--muted);
  font-weight: 650;
}

.mini-check {
  border: 1px solid var(--line);
  border-radius: 999px;
  padding: 7px 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  line-height: 1.35;
  background: rgba(255, 255, 255, 0.78);
  transition: background 0.16s ease, transform 0.16s ease;
}

.mini-check:hover {
  background: #fff;
  transform: translateY(-1px);
}

.qr-card {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: 14px;
}

.qr-box {
  display: flex;
  justify-content: center;
  align-items: center;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  background: #fff;
  padding: 14px;
}

.qr-box img { width: 240px; height: 240px; }
.qr-info textarea { width: 100%; min-height: 76px; }
.qr-info .label { font-size: 12px; color: var(--muted); margin: 0; }
.qr-info .status { margin: 8px 0 0; color: #15803d; font-size: 13px; }
.status { margin: 0; color: #15803d; font-size: 13px; }
.status-error { color: var(--danger); }

.status-banner {
  border: 1px solid rgba(52, 199, 89, 0.20);
  background: rgba(240, 253, 244, 0.82);
  color: #15803d;
  border-radius: 18px;
  padding: 12px 14px;
  box-shadow: 0 14px 34px rgba(21, 128, 61, 0.08);
}

.status-banner.status-error {
  border-color: rgba(255, 59, 48, 0.20);
  background: rgba(255, 241, 242, 0.88);
  color: #b42318;
}

.toast-layer {
  position: fixed;
  top: 18px;
  left: 0;
  right: 0;
  z-index: 1000;
  display: flex;
  justify-content: center;
  pointer-events: none;
  padding: 0 16px;
}

.toast {
  margin: 0;
  width: fit-content;
  max-width: min(560px, calc(100vw - 32px));
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 700;
  pointer-events: auto;
  backdrop-filter: blur(18px);
}

@media (max-width: 980px) {
  .page-header { align-items: flex-start; flex-direction: column; }
  .stat-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .qr-card { grid-template-columns: 1fr; }
  .toolbar .input { flex-basis: 100%; }
}

@media (max-width: 640px) {
  .stat-grid { grid-template-columns: 1fr; }
}
"#;
