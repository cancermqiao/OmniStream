pub const CSS: &str = r#"
.card {
  background: var(--panel-strong);
  border: 1px solid rgba(255, 255, 255, 0.76);
  border-radius: 24px;
  padding: 14px;
  box-shadow: 0 18px 54px rgba(15, 23, 42, 0.08);
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
  padding: 12px 9px;
  border-bottom: 1px solid rgba(17, 24, 39, 0.06);
  vertical-align: middle;
  min-width: 0;
}

.table th {
  color: var(--muted);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  background: rgba(248, 250, 252, 0.78);
}

.table tr:hover td { background: rgba(255, 255, 255, 0.74); }

.table-downloads th:nth-child(1),
.table-downloads td:nth-child(1) { width: 48px; }
.table-downloads th:nth-child(2),
.table-downloads td:nth-child(2) { width: 16%; }
.table-downloads th:nth-child(3),
.table-downloads td:nth-child(3) { width: 25%; }
.table-downloads th:nth-child(4),
.table-downloads td:nth-child(4) { width: 12%; }
.table-downloads th:nth-child(5),
.table-downloads td:nth-child(5) { width: 18%; }

.table-uploads th:nth-child(1),
.table-uploads td:nth-child(1) { width: 46px; }
.table-uploads th:nth-child(2),
.table-uploads td:nth-child(2) { width: 12%; }
.table-uploads th:nth-child(3),
.table-uploads td:nth-child(3) { width: 12%; }
.table-uploads th:nth-child(4),
.table-uploads td:nth-child(4) { width: 24%; }
.table-uploads th:nth-child(5),
.table-uploads td:nth-child(5) { width: 10%; }
.table-uploads th:nth-child(6),
.table-uploads td:nth-child(6) { width: 12%; }
.table-uploads th:nth-child(7),
.table-uploads td:nth-child(7) { width: 16%; }

.table .actions {
  width: 240px;
  min-width: 0;
  text-align: right;
  white-space: normal;
}

.table .actions .btn {
  margin: 2px 0 2px 4px;
  padding: 6px 9px;
  font-size: 11px;
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
  padding: 3px 8px;
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
  display: block;
  max-width: 100%;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title-cell {
  overflow: hidden;
}

@media (max-width: 980px) {
  .table { table-layout: auto; }
  .table-wrap { overflow-x: auto; }
  .table .actions { width: auto; }
}
"#;
