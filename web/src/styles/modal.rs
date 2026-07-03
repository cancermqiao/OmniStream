pub const CSS: &str = r#"
.modal-wrap {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
  overflow: hidden;
}

.modal-mask {
  position: absolute;
  inset: 0;
  background: rgba(10, 16, 29, 0.38);
  backdrop-filter: blur(20px);
}

.modal {
  position: relative;
  width: min(700px, calc(100vw - 36px));
  max-width: 100%;
  max-height: min(90vh, 900px);
  overflow: auto;
  background: rgba(255, 255, 255, 0.90);
  border: 1px solid rgba(255, 255, 255, 0.80);
  border-radius: 24px;
  padding: 22px 24px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 30px 100px rgba(15, 23, 42, 0.22);
  backdrop-filter: blur(24px);
  overscroll-behavior: contain;
}

.modal.wide { width: min(860px, calc(100vw - 36px)); }
.modal h3 {
  margin: 0 0 2px;
  font-size: 22px;
  line-height: 1.12;
  font-weight: 760;
  letter-spacing: -0.045em;
}
.section-title {
  margin: 10px 0 0;
  font-size: 12px;
  font-weight: 700;
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
  .modal-wrap { align-items: stretch; padding: 12px; }
  .modal,
  .modal.wide {
    width: calc(100vw - 24px);
    max-height: calc(100vh - 24px);
    padding: 20px;
  }
}
"#;
