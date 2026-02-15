pub const CSS: &str = r#"
.modal-wrap {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-mask {
  position: absolute;
  inset: 0;
  background: rgba(15, 23, 42, 0.4);
}

.modal {
  position: relative;
  width: min(680px, 94vw);
  background: #fff;
  border: 1px solid #dbe3ef;
  border-radius: 10px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.modal.wide { width: min(820px, 94vw); }
.modal h3 { margin: 0; font-size: 20px; }
.section-title {
  margin: 4px 0 0;
  font-size: 12px;
  font-weight: 700;
  color: #334155;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.check-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.check-item {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

@media (max-width: 980px) {
  .check-grid { grid-template-columns: 1fr; }
}
"#;
