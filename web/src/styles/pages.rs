pub const CSS: &str = r#"
.page { display: flex; flex-direction: column; gap: 14px; }

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 12px;
  padding: 4px 0 2px;
}

.page-header h1 {
  margin: 0;
  font-size: 24px;
  letter-spacing: -0.02em;
}

.page-header p {
  margin: 4px 0 0;
  color: #64748b;
  font-size: 13px;
}

.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}

.stat-card {
  background: rgba(255, 255, 255, 0.86);
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  padding: 12px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.06);
}

.stat-label {
  margin: 0;
  color: #64748b;
  font-size: 12px;
  font-weight: 700;
}

.stat-value {
  margin: 5px 0 0;
  color: #0f172a;
  font-size: 24px;
  font-weight: 800;
  letter-spacing: -0.04em;
}

.stat-hint {
  margin: 4px 0 0;
  color: #94a3b8;
  font-size: 11px;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.toolbar + .toolbar {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed #e2e8f0;
}

.toolbar .input {
  min-width: 260px;
  max-width: 360px;
}

.toolbar-label {
  font-size: 12px;
  color: #64748b;
  font-weight: 600;
}

.mini-check {
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 4px 7px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  background: #fff;
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
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  background: #fff;
  padding: 10px;
}

.qr-box img { width: 240px; height: 240px; }
.qr-info textarea { width: 100%; min-height: 76px; }
.qr-info .label { font-size: 12px; color: #64748b; margin: 0; }
.qr-info .status { margin: 8px 0 0; color: #0f766e; font-size: 13px; }
.status { margin: 0; color: #0f766e; font-size: 13px; }
.status-error { color: #be123c; }

.status-banner {
  border: 1px solid #99f6e4;
  background: #f0fdfa;
  color: #0f766e;
  border-radius: 12px;
  padding: 10px 12px;
  box-shadow: 0 10px 22px rgba(15, 118, 110, 0.08);
}

.status-banner.status-error {
  border-color: #fecdd3;
  background: #fff1f2;
  color: #be123c;
}

@media (max-width: 980px) {
  .page-header { align-items: flex-start; flex-direction: column; }
  .stat-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .qr-card { grid-template-columns: 1fr; }
}

@media (max-width: 640px) {
  .stat-grid { grid-template-columns: 1fr; }
}
"#;
