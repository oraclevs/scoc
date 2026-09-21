#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def main() -> int:
    request = json.load(sys.stdin)
    import jc
    import jc.lib

    version = getattr(jc, "__version__", getattr(jc.lib, "__version__", None))
    if version != "1.26.0":
        print(f"expected jc 1.26.0, found {version!r}", file=sys.stderr)
        return 2

    parser = request["parser"]
    if request.get("streaming"):
        parser = request.get("streaming_parser") or f"{parser}-s"
    data = Path(request["input_path"]).read_text(encoding="utf-8")
    kwargs = {"raw": bool(request.get("raw", False)), "quiet": True}
    if request.get("streaming"):
        kwargs["ignore_exceptions"] = bool(request.get("ignore_errors", False))
        result = list(jc.parse(parser, data.splitlines(), **kwargs))
    else:
        result = jc.parse(parser, data, **kwargs)
    json.dump(result, sys.stdout, separators=(",", ":"), ensure_ascii=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
