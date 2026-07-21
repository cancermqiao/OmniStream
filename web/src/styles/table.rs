pub const CSS: &str = r#"
.card {
  background: var(--panel-strong);
  border: 1px solid rgba(255, 255, 255, 0.80);
  border-radius: 22px;
  padding: 12px;
  box-shadow: 0 16px 46px rgba(15, 23, 42, 0.075);
  backdrop-filter: blur(24px);
  min-width: 0;
  max-width: 100%;
}

.table-wrap {
  width: 100%;
  max-width: 100%;
  overflow-x: clip;
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
  padding: 11px 8px;
  border-bottom: 1px solid rgba(17, 24, 39, 0.06);
  vertical-align: middle;
  min-width: 0;
  overflow: hidden;
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
.table-downloads td:nth-child(1) { width: 42px; }
.table-downloads th:nth-child(2),
.table-downloads td:nth-child(2) { width: 12%; }
.table-downloads th:nth-child(3),
.table-downloads td:nth-child(3) { width: 28%; }
.table-downloads th:nth-child(4),
.table-downloads td:nth-child(4) { width: 9%; }
.table-downloads th:nth-child(5),
.table-downloads td:nth-child(5) { width: 9%; }
.table-downloads th:nth-child(6),
.table-downloads td:nth-child(6) { width: 10%; }

.table-uploads th:nth-child(1),
.table-uploads td:nth-child(1) { width: 42px; }
.table-uploads th:nth-child(2),
.table-uploads td:nth-child(2) { width: 12%; }
.table-uploads th:nth-child(3),
.table-uploads td:nth-child(3) { width: 11%; }
.table-uploads th:nth-child(4),
.table-uploads td:nth-child(4) { width: 23%; }
.table-uploads th:nth-child(5),
.table-uploads td:nth-child(5) { width: 9%; }
.table-uploads th:nth-child(6),
.table-uploads td:nth-child(6) { width: 11%; }
.table-uploads th:nth-child(7),
.table-uploads td:nth-child(7) { width: 13%; }

.table .actions {
  width: 182px;
  min-width: 0;
  text-align: right;
  white-space: normal;
  overflow: visible;
}

.table .actions .btn {
  margin: 2px 0 2px 3px;
  padding: 5px 7px;
  font-size: 10px;
  line-height: 1.25;
}

.empty {
  text-align: center;
  color: var(--muted);
  padding: 30px;
}

.tag {
  display: inline-block;
  margin: 0 3px 4px 0;
  border-radius: 999px;
  padding: 3px 7px;
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

.storage-size {
  color: var(--ink);
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
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

.url-cell {
  overflow: hidden;
}

.table td:not(.actions):not(.title-cell) {
  text-overflow: ellipsis;
}

@media (max-width: 980px) {
  .table { table-layout: auto; }
  .table-wrap { overflow-x: auto; }
  .table .actions { width: auto; }
}

@media (max-width: 640px) {
  .card {
    border-radius: 18px;
    padding: 8px;
  }

  .table-wrap {
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    border-radius: 14px;
  }

  .table {
    min-width: 720px;
    font-size: 12px;
  }

  .table-downloads {
    min-width: 840px;
  }

  .table-uploads {
    min-width: 860px;
  }

  .table th,
  .table td {
    padding: 9px 7px;
  }

  .table th {
    font-size: 9.5px;
    letter-spacing: 0.04em;
  }

  .table .actions {
    min-width: 136px;
    text-align: left;
  }

  .table .actions .btn {
    margin: 2px 3px 2px 0;
    padding: 6px 8px;
    font-size: 10.5px;
    min-height: 30px;
  }

  .tag {
    padding: 4px 7px;
    font-size: 10.5px;
  }

  .empty {
    padding: 22px 12px;
  }
}

@media (max-width: 480px) {
  .table-wrap::after {
    content: "左右滑动查看更多";
    display: block;
    margin: 8px 2px 0;
    color: var(--soft);
    font-size: 11px;
    text-align: center;
  }
}
"#;
