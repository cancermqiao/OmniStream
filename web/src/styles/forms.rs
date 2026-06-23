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
  white-space: nowrap;
  transition: transform 0.14s ease, box-shadow 0.14s ease, border-color 0.14s ease;
}

.btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 8px 18px rgba(15, 23, 42, 0.08);
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.48;
}

.btn-primary {
  background: #0f766e;
  color: #fff;
  border-color: #0f766e;
}

.btn-warning {
  background: #fffbeb;
  color: #b45309;
  border-color: #fde68a;
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
  outline: none;
  transition: border-color 0.14s ease, box-shadow 0.14s ease;
}

.input:focus {
  border-color: #14b8a6;
  box-shadow: 0 0 0 3px rgba(20, 184, 166, 0.13);
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
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 980px) {
  .grid-2 { grid-template-columns: 1fr; }
}
"#;
