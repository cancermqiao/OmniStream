pub const CSS: &str = r#"
.card {
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  padding: 12px;
  box-shadow: 0 14px 30px rgba(15, 23, 42, 0.06);
}

.table-wrap {
  width: 100%;
  overflow-x: auto;
}

.table {
  width: 100%;
  border-collapse: collapse;
  min-width: 760px;
}

.table th, .table td {
  text-align: left;
  padding: 7px 8px;
  border-bottom: 1px solid #eef2f7;
  vertical-align: middle;
}

.table th {
  color: #64748b;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  background: #f8fafc;
}

.table tr:hover td { background: #f8fafc; }

.table .actions {
  width: 165px;
  text-align: right;
  white-space: nowrap;
}

.empty {
  text-align: center;
  color: #64748b;
  padding: 20px;
}

.tag {
  display: inline-block;
  margin: 0 4px 4px 0;
  border-radius: 999px;
  padding: 2px 7px;
  background: #f1f5f9;
  color: #334155;
  font-size: 11px;
  border: 1px solid #e2e8f0;
}

.tag-success {
  background: #ecfdf5;
  color: #047857;
  border-color: #bbf7d0;
}

.tag-warning {
  background: #fffbeb;
  color: #b45309;
  border-color: #fde68a;
}

.tag-danger {
  background: #fff1f2;
  color: #be123c;
  border-color: #fecdd3;
}

.tag-info {
  background: #eff6ff;
  color: #1d4ed8;
  border-color: #bfdbfe;
}

.muted {
  color: #94a3b8;
  font-size: 12px;
}

.text-ellipsis {
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 980px) {
  .table .actions { width: auto; }
}
"#;
