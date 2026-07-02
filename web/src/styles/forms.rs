pub const CSS: &str = r#"
.btn {
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.82);
  color: var(--ink);
  border-radius: 999px;
  padding: 8px 13px;
  cursor: pointer;
  font-weight: 800;
  font-size: 12px;
  margin-left: 6px;
  white-space: nowrap;
  transition: transform 0.16s ease, box-shadow 0.16s ease, border-color 0.16s ease, background 0.16s ease;
}

.btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 14px 28px rgba(15, 23, 42, 0.10);
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.48;
}

.btn-primary {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
  box-shadow: 0 12px 26px rgba(0, 122, 255, 0.22);
}

.btn-warning {
  background: rgba(255, 251, 235, 0.88);
  color: #9a5b00;
  border-color: rgba(245, 158, 11, 0.24);
}

.btn-danger {
  background: rgba(255, 241, 242, 0.88);
  color: #b42318;
  border-color: rgba(255, 59, 48, 0.24);
}

.btn-ghost {
  background: rgba(255, 255, 255, 0.58);
}

.input {
  width: 100%;
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 10px 12px;
  font: inherit;
  color: inherit;
  font-size: 13px;
  outline: none;
  background: rgba(255, 255, 255, 0.82);
  transition: border-color 0.16s ease, box-shadow 0.16s ease, background 0.16s ease;
}

.input:focus {
  background: #fff;
  border-color: rgba(0, 122, 255, 0.64);
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.13);
}

.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: 12px; color: #374151; font-weight: 800; }
.label { color: var(--muted); font-size: 12px; line-height: 1.55; margin: 6px 0 0; }
.section-title {
  margin: 18px 0 10px;
  color: var(--ink);
  font-size: 13px;
  font-weight: 800;
  letter-spacing: -0.01em;
}

.grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.inline-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 980px) {
  .grid-2 { grid-template-columns: 1fr; }
}
"#;
