pub const CSS: &str = r#"
.btn {
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.82);
  color: var(--ink);
  border-radius: 999px;
  padding: 8px 14px;
  cursor: pointer;
  font-weight: 650;
  font-size: 12px;
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
  border-radius: 15px;
  padding: 11px 15px;
  font: inherit;
  color: inherit;
  font-size: 13px;
  outline: none;
  background: rgba(255, 255, 255, 0.82);
  transition: border-color 0.16s ease, box-shadow 0.16s ease, background 0.16s ease;
}

select.input {
  appearance: none;
  padding-right: 40px;
  background-image:
    linear-gradient(45deg, transparent 50%, #6b7280 50%),
    linear-gradient(135deg, #6b7280 50%, transparent 50%);
  background-position:
    calc(100% - 20px) calc(50% - 2px),
    calc(100% - 15px) calc(50% - 2px);
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
}

textarea.input {
  min-height: 96px;
  resize: vertical;
  line-height: 1.55;
}

.input:focus {
  background: #fff;
  border-color: rgba(0, 122, 255, 0.64);
  box-shadow: 0 0 0 4px rgba(0, 122, 255, 0.13);
}

.field { display: flex; flex-direction: column; gap: 7px; min-width: 0; }
.field label { font-size: 12px; color: #374151; font-weight: 650; }
.label { color: var(--muted); font-size: 12px; line-height: 1.55; margin: 6px 0 0; }
.section-title {
  margin: 18px 0 10px;
  color: var(--ink);
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  min-width: 0;
}

.inline-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
}

.select-panel,
.tag-editor {
  border: 1px solid var(--line);
  border-radius: 20px;
  padding: 10px;
  background: rgba(255, 255, 255, 0.64);
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.select-panel .input,
.tag-editor .input {
  background: rgba(255, 255, 255, 0.88);
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  min-height: 28px;
  min-width: 0;
}

.chip {
  border: 1px solid rgba(0, 122, 255, 0.16);
  border-radius: 999px;
  background: rgba(239, 246, 255, 0.82);
  color: #1d4ed8;
  padding: 6px 10px;
  font-size: 12px;
  font-weight: 650;
  line-height: 1;
}

.chip-removable {
  cursor: pointer;
  transition: transform 0.16s ease, background 0.16s ease;
}

.chip-removable:hover {
  background: rgba(219, 234, 254, 0.95);
  transform: translateY(-1px);
}

.option-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 220px;
  overflow: auto;
  padding-right: 2px;
}

.option-row {
  width: 100%;
  border: 1px solid transparent;
  border-radius: 14px;
  padding: 10px 12px;
  background: transparent;
  color: var(--ink);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  text-align: left;
  font: inherit;
  font-size: 13px;
  transition: background 0.16s ease, border-color 0.16s ease, transform 0.16s ease;
}

.option-row:hover {
  background: rgba(255, 255, 255, 0.84);
  border-color: rgba(17, 24, 39, 0.07);
}

.option-row-active {
  background: rgba(0, 122, 255, 0.10);
  border-color: rgba(0, 122, 255, 0.20);
}

.option-state {
  flex: 0 0 auto;
  color: var(--accent);
  font-size: 12px;
  font-weight: 650;
}

.empty-option {
  margin: 0;
  padding: 10px 12px;
  color: var(--soft);
  font-size: 12px;
}

.segmented {
  display: inline-flex;
  width: fit-content;
  max-width: 100%;
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--line);
  border-radius: 999px;
  background: rgba(243, 244, 246, 0.72);
}

.segment {
  min-width: 88px;
  border: 0;
  border-radius: 999px;
  padding: 8px 14px;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 650;
  transition: background 0.16s ease, color 0.16s ease, box-shadow 0.16s ease;
}

.segment-active {
  background: #fff;
  color: var(--ink);
  box-shadow: 0 10px 24px rgba(15, 23, 42, 0.10);
}

@media (max-width: 980px) {
  .grid-2 { grid-template-columns: 1fr; }
}
"#;
