pub const CSS: &str = r#"
.card {
  background: var(--panel-strong);
  border: 1px solid rgba(255, 255, 255, 0.76);
  border-radius: var(--radius-lg);
  padding: 16px;
  box-shadow: var(--shadow-soft);
  backdrop-filter: blur(24px);
  min-width: 0;
}

.table-wrap {
  width: 100%;
  max-width: 100%;
  overflow-x: hidden;
  min-width: 0;
}

.table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  min-width: 0;
  table-layout: fixed;
}

.table th, .table td {
  text-align: left;
  padding: 13px 10px;
  border-bottom: 1px solid rgba(17, 24, 39, 0.06);
  vertical-align: middle;
  overflow-wrap: anywhere;
}

.table th {
  color: var(--muted);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  background: rgba(248, 250, 252, 0.78);
}

.table tr:hover td { background: rgba(255, 255, 255, 0.74); }

.table .actions {
  width: 300px;
  min-width: 0;
  text-align: right;
  white-space: normal;
}

.table .actions .btn {
  margin: 2px 0 2px 6px;
  padding: 7px 11px;
}

.empty {
  text-align: center;
  color: var(--muted);
  padding: 30px;
}

.tag {
  display: inline-block;
  margin: 0 4px 4px 0;
  border-radius: 999px;
  padding: 4px 9px;
  background: rgba(243, 244, 246, 0.92);
  color: #374151;
  font-size: 11px;
  font-weight: 650;
  border: 1px solid rgba(17, 24, 39, 0.07);
}

.tag-success {
  background: rgba(236, 253, 245, 0.92);
  color: #047857;
  border-color: rgba(34, 197, 94, 0.22);
}

.tag-warning {
  background: rgba(255, 251, 235, 0.94);
  color: #9a5b00;
  border-color: rgba(245, 158, 11, 0.25);
}

.tag-danger {
  background: rgba(255, 241, 242, 0.94);
  color: #b42318;
  border-color: rgba(255, 59, 48, 0.25);
}

.tag-info {
  background: rgba(239, 246, 255, 0.94);
  color: #1d4ed8;
  border-color: rgba(0, 122, 255, 0.20);
}

.muted {
  color: var(--soft);
  font-size: 12px;
}

.text-ellipsis {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 980px) {
  .table { table-layout: auto; }
  .table-wrap { overflow-x: auto; }
  .table .actions { width: auto; }
}
"#;
