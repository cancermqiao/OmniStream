pub const CSS: &str = r#"
.card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 8px;
}

.table {
  width: 100%;
  border-collapse: collapse;
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
}

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

@media (max-width: 980px) {
  .table .actions { width: auto; }
}
"#;
