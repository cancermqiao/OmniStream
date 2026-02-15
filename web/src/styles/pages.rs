pub const CSS: &str = r#"
.page { display: flex; flex-direction: column; gap: 10px; }

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 12px;
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

@media (max-width: 980px) {
  .qr-card { grid-template-columns: 1fr; }
}
"#;
