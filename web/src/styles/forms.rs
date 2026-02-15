pub const CSS: &str = r#"
.btn {
  border: 1px solid #dbe3ef;
  background: #fff;
  color: #0f172a;
  border-radius: 8px;
  padding: 6px 10px;
  cursor: pointer;
  font-weight: 600;
  font-size: 12px;
  margin-left: 6px;
}

.btn-primary {
  background: #0f766e;
  color: #fff;
  border-color: #0f766e;
}

.btn-danger {
  background: #fff1f2;
  color: #be123c;
  border-color: #fecdd3;
}

.btn-ghost {
  background: #f8fafc;
}

.input {
  width: 100%;
  border: 1px solid #dbe3ef;
  border-radius: 8px;
  padding: 7px 9px;
  font: inherit;
  color: inherit;
  font-size: 13px;
}

.field { display: flex; flex-direction: column; gap: 6px; }
.field label { font-size: 12px; color: #475569; font-weight: 600; }

.grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.inline-actions {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 980px) {
  .grid-2 { grid-template-columns: 1fr; }
}
"#;
