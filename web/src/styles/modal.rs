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
  background: rgba(15, 23, 42, 0.34);
  backdrop-filter: blur(18px);
}

.modal {
  position: relative;
  width: min(680px, 94vw);
  max-height: min(88vh, 900px);
  overflow: auto;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.78);
  border-radius: 28px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-shadow: 0 34px 120px rgba(15, 23, 42, 0.22);
  backdrop-filter: blur(24px);
}

.modal.wide { width: min(820px, 94vw); }
.modal h3 { margin: 0 0 4px; font-size: 24px; letter-spacing: -0.04em; }
.section-title {
  margin: 10px 0 0;
  font-size: 12px;
  font-weight: 800;
  color: var(--ink);
  text-transform: uppercase;
  letter-spacing: 0.07em;
}

.check-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.check-item {
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.62);
}

@media (max-width: 980px) {
  .check-grid { grid-template-columns: 1fr; }
}
"#;
