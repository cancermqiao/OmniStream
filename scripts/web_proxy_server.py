#!/usr/bin/env python3
import argparse
import http.client
import os
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit


class ProxyStaticHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, directory: str, api_host: str, api_port: int, **kwargs):
        self.api_host = api_host
        self.api_port = api_port
        super().__init__(*args, directory=directory, **kwargs)

    def do_GET(self):
        if self._is_api():
            self._proxy()
            return
        super().do_GET()

    def do_POST(self):
        if self._is_api():
            self._proxy()
            return
        self.send_error(405, "Unsupported method")

    def do_PUT(self):
        if self._is_api():
            self._proxy()
            return
        self.send_error(405, "Unsupported method")

    def do_PATCH(self):
        if self._is_api():
            self._proxy()
            return
        self.send_error(405, "Unsupported method")

    def do_DELETE(self):
        if self._is_api():
            self._proxy()
            return
        self.send_error(405, "Unsupported method")

    def do_OPTIONS(self):
        if self._is_api():
            self._proxy()
            return
        self.send_response(204)
        self.end_headers()

    def _is_api(self) -> bool:
        path = self.path
        return path == "/api" or path.startswith("/api/")

    def _proxy(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length) if length > 0 else None
        split = urlsplit(self.path)
        target = split.path
        if split.query:
            target = f"{target}?{split.query}"

        conn = http.client.HTTPConnection(self.api_host, self.api_port, timeout=30)
        try:
            headers = {
                k: v
                for k, v in self.headers.items()
                if k.lower() not in {"host", "connection", "content-length"}
            }
            headers["Host"] = f"{self.api_host}:{self.api_port}"
            conn.request(self.command, target, body=body, headers=headers)
            upstream = conn.getresponse()
            payload = upstream.read()

            self.send_response(upstream.status, upstream.reason)
            for k, v in upstream.getheaders():
                kl = k.lower()
                if kl in {"transfer-encoding", "connection", "keep-alive"}:
                    continue
                self.send_header(k, v)
            self.end_headers()
            if payload:
                self.wfile.write(payload)
        except Exception as exc:
            self.send_response(502, "Bad Gateway")
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.end_headers()
            self.wfile.write(f"proxy error: {exc}\n".encode("utf-8"))
        finally:
            conn.close()


def main():
    parser = argparse.ArgumentParser(description="Serve static files and proxy /api")
    parser.add_argument("--web-dir", required=True, help="Directory containing web assets")
    parser.add_argument("--web-port", type=int, default=8080)
    parser.add_argument("--api-host", default="127.0.0.1")
    parser.add_argument("--api-port", type=int, default=3000)
    args = parser.parse_args()

    web_dir = os.path.abspath(args.web_dir)
    if not os.path.isdir(web_dir):
        raise SystemExit(f"web dir not found: {web_dir}")

    def handler(*h_args, **h_kwargs):
        return ProxyStaticHandler(
            *h_args,
            directory=web_dir,
            api_host=args.api_host,
            api_port=args.api_port,
            **h_kwargs,
        )

    server = ThreadingHTTPServer(("0.0.0.0", args.web_port), handler)
    print(
        f"serving web={web_dir} on :{args.web_port}, proxy /api -> {args.api_host}:{args.api_port}",
        flush=True,
    )
    server.serve_forever()


if __name__ == "__main__":
    main()
