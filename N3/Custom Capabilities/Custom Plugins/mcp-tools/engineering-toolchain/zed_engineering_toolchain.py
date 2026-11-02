#!/usr/bin/env python3
"""Bounded stdio MCP server for reproducible, language-aware validation.

The server selects only declared commands. It never executes an agent-provided
shell string, and it returns the exact command, exit status, timeout state, and
bounded output so a passing claim always has an inspectable receipt.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

MAX_OUTPUT = 48_000
MAX_TIMEOUT = 900
SERVER_NAME = "zed-engineering-toolchain"
IGNORED = {".git", "node_modules", "target", "build", ".next", "DerivedData", ".swiftpm"}
LANGUAGES = ("rust", "swift", "cpp", "metal", "typescript", "web", "tauri", "wasm", "gpui", "mlx", "python", "java")


def emit(request_id: Any, payload: dict[str, Any]) -> None:
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": payload}), flush=True)


def tool_result(payload: Any, failed: bool = False) -> dict[str, Any]:
    return {"content": [{"type": "text", "text": json.dumps(payload, indent=2, sort_keys=True)}], "isError": failed}


def root_from(value: str | None) -> Path:
    root = Path(value or os.getcwd()).expanduser().resolve()
    if not root.is_dir():
        raise ValueError(f"workspace is not a directory: {root}")
    return root


def available(command: str) -> bool:
    return shutil.which(command) is not None


def has_any(root: Path, names: tuple[str, ...]) -> bool:
    return any((root / name).exists() for name in names)


def contains_suffix(root: Path, suffix: str, limit: int = 64) -> bool:
    seen = 0
    for current, directories, files in os.walk(root):
        directories[:] = [item for item in directories if item not in IGNORED]
        for file in files:
            if file.endswith(suffix):
                return True
            seen += 1
            if seen >= limit * 100:
                return False
    return False


def package_scripts(root: Path) -> set[str]:
    try:
        package = json.loads((root / "package.json").read_text(encoding="utf-8"))
        scripts = package.get("scripts", {})
        return {name for name, command in scripts.items() if isinstance(name, str) and isinstance(command, str)}
    except (FileNotFoundError, json.JSONDecodeError):
        return set()


def is_detected(root: Path, language: str, markers: tuple[str, ...]) -> bool:
    if has_any(root, markers):
        return True
    if language == "metal":
        return contains_suffix(root, ".metal")
    if language == "gpui":
        manifest = root / "Cargo.toml"
        return manifest.is_file() and "gpui" in manifest.read_text(encoding="utf-8", errors="ignore")
    return False


def detect(arguments: dict[str, Any]) -> dict[str, Any]:
    root = root_from(arguments.get("workspace"))
    scripts = package_scripts(root)
    markers = {
        "rust": ("Cargo.toml",),
        "swift": ("Package.swift",),
        "cpp": ("CMakeLists.txt", "meson.build", "compile_commands.json"),
        "metal": (),
        "typescript": ("package.json", "tsconfig.json"),
        "web": ("package.json", "next.config.js", "next.config.ts", "tailwind.config.js", "tailwind.config.ts", "components.json"),
        "tauri": ("src-tauri/tauri.conf.json", "src-tauri/tauri.conf.json5", "src-tauri/Cargo.toml"),
        "wasm": ("wasm-pack.toml",),
        "gpui": (),
        "mlx": ("requirements.txt", "pyproject.toml"),
        "python": ("pyproject.toml", "requirements.txt", "setup.py"),
        "java": ("pom.xml", "build.gradle", "build.gradle.kts", "gradlew"),
    }
    tools = {
        "rust": ("cargo", "rustfmt", "clippy-driver"), "swift": ("swift", "swift-format"),
        "cpp": ("cmake", "ctest", "clang++"), "metal": ("xcrun",),
        "typescript": ("node", "npm"), "web": ("node", "npm"), "tauri": ("cargo", "npm"),
        "wasm": ("cargo", "wasm-pack"), "gpui": ("cargo",), "mlx": ("python3",),
        "python": ("python3", "ruff", "pytest"), "java": ("java", "javac", "mvn"),
    }
    return {
        "workspace": str(root), "package_scripts": sorted(scripts),
        "languages": {name: {"detected": is_detected(root, name, found), "markers": [item for item in found if (root / item).exists()] + (["*.metal"] if name == "metal" and contains_suffix(root, ".metal") else []), "executables": {item: available(item) for item in tools[name]}} for name, found in markers.items()},
    }


def npm_plan(root: Path, operation: str) -> list[str]:
    scripts = package_scripts(root)
    aliases = {
        "check": ("typecheck", "check"), "typecheck": ("typecheck",), "lint": ("lint",),
        "test": ("test",), "build": ("build",), "e2e": ("test:e2e", "e2e"),
    }
    for candidate in aliases.get(operation, ()):
        if candidate in scripts:
            return ["npm", "run", candidate]
    raise ValueError(f"no package.json script supports {operation}; available: {', '.join(sorted(scripts)) or 'none'}")


def plan(root: Path, language: str, operation: str) -> list[str]:
    if language not in LANGUAGES:
        raise ValueError(f"unsupported language: {language}")
    static: dict[str, dict[str, list[str]]] = {
        "rust": {"format": ["cargo", "fmt", "--check"], "check": ["cargo", "check", "--all-targets"], "lint": ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], "test": ["cargo", "test", "--all-targets"]},
        "swift": {"format": ["swift-format", "lint", "-r", "."], "check": ["swift", "build"], "build": ["swift", "build"], "test": ["swift", "test"]},
        "cpp": {"configure": ["cmake", "-S", ".", "-B", "build"], "build": ["cmake", "--build", "build"], "test": ["ctest", "--test-dir", "build", "--output-on-failure"]},
        "mlx": {"import": ["python3", "-c", "import mlx.core as mx; print(mx.default_device())"], "test": ["python3", "-m", "pytest", "-q"]},
        "python": {"syntax": ["python3", "-m", "compileall", "-q", "."], "format": ["ruff", "format", "--check", "."], "lint": ["ruff", "check", "."], "test": ["python3", "-m", "pytest", "-q"]},
        "wasm": {"check": ["cargo", "check", "--target", "wasm32-unknown-unknown"], "test": ["cargo", "test"]},
        "gpui": {"check": ["cargo", "check", "--all-targets"], "test": ["cargo", "test", "--all-targets"]},
    }
    if language in {"typescript", "web"}:
        return npm_plan(root, operation)
    if language == "tauri":
        commands = {"check": ["cargo", "check", "--manifest-path", "src-tauri/Cargo.toml"], "test": ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml"], "build": ["cargo", "build", "--manifest-path", "src-tauri/Cargo.toml"]}
        return commands[operation]
    if language == "java":
        if (root / "mvnw").exists():
            return ["./mvnw", {"check": "verify", "test": "test", "build": "package"}[operation]]
        if (root / "gradlew").exists():
            return ["./gradlew", {"check": "check", "test": "test", "build": "build"}[operation]]
        if (root / "pom.xml").exists():
            return ["mvn", {"check": "verify", "test": "test", "build": "package"}[operation]]
        raise ValueError("Java requires pom.xml, mvnw, or gradlew; direct javac is intentionally not guessed")
    if language == "metal":
        sources = []
        for current, directories, files in os.walk(root):
            directories[:] = [item for item in directories if item not in IGNORED]
            sources.extend(Path(current) / item for item in files if item.endswith(".metal"))
        if not sources:
            raise ValueError("Metal validation requires a .metal file")
        if operation != "syntax":
            raise ValueError("Metal supports syntax only; compilation writes only to a temporary directory")
        return ["xcrun", "--sdk", "macosx", "metal", "-fsyntax-only", str(sources[0])]
    try:
        return static[language][operation]
    except KeyError as error:
        raise ValueError(f"unsupported {language} operation: {operation}") from error


def run(arguments: dict[str, Any]) -> dict[str, Any]:
    root = root_from(arguments.get("workspace"))
    language, operation = arguments.get("language"), arguments.get("operation")
    timeout = int(arguments.get("timeout_seconds", 300))
    if not isinstance(language, str) or not isinstance(operation, str):
        raise ValueError("language and operation are required strings")
    if not 1 <= timeout <= MAX_TIMEOUT:
        raise ValueError(f"timeout_seconds must be between 1 and {MAX_TIMEOUT}")
    command = plan(root, language, operation)
    executable = root / command[0]
    if not available(command[0]) and not executable.exists():
        raise ValueError(f"required executable is unavailable: {command[0]}")
    try:
        completed = subprocess.run(command, cwd=root, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=timeout, check=False)
        return {"workspace": str(root), "language": language, "operation": operation, "command": command, "exit_code": completed.returncode, "timed_out": False, "output": completed.stdout[-MAX_OUTPUT:], "truncated": len(completed.stdout) > MAX_OUTPUT}
    except subprocess.TimeoutExpired as error:
        output = error.stdout or ""
        return {"workspace": str(root), "language": language, "operation": operation, "command": command, "exit_code": None, "timed_out": True, "output": output[-MAX_OUTPUT:]}


TOOLS = [
    {"name": "detect_toolchains", "description": "Inspect a workspace for Rust, Swift, C/C++, Metal, TypeScript/web, Tauri, WASM, GPUI, MLX, Python, and Java markers and required executables.", "inputSchema": {"type": "object", "properties": {"workspace": {"type": "string"}}}},
    {"name": "plan_language_check", "description": "Return the exact allowlisted validation command without running it.", "inputSchema": {"type": "object", "required": ["language", "operation"], "properties": {"workspace": {"type": "string"}, "language": {"type": "string", "enum": list(LANGUAGES)}, "operation": {"type": "string"}}}},
    {"name": "run_language_check", "description": "Run one allowlisted validation command with a bounded timeout; arbitrary shell commands are never accepted.", "inputSchema": {"type": "object", "required": ["language", "operation"], "properties": {"workspace": {"type": "string"}, "language": {"type": "string", "enum": list(LANGUAGES)}, "operation": {"type": "string"}, "timeout_seconds": {"type": "integer", "minimum": 1, "maximum": MAX_TIMEOUT}}}},
]


def dispatch(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    if name == "detect_toolchains": return detect(arguments)
    if name == "plan_language_check": return {"workspace": str(root_from(arguments.get("workspace"))), "command": plan(root_from(arguments.get("workspace")), arguments["language"], arguments["operation"])}
    if name == "run_language_check": return run(arguments)
    raise ValueError(f"unknown tool: {name}")


def main() -> None:
    for raw in sys.stdin:
        try:
            request = json.loads(raw); method = request.get("method"); request_id = request.get("id")
            if method == "notifications/initialized": continue
            if method == "initialize": emit(request_id, {"protocolVersion": request.get("params", {}).get("protocolVersion", "2025-03-26"), "capabilities": {"tools": {"listChanged": False}}, "serverInfo": {"name": SERVER_NAME, "version": "2.0.0"}})
            elif method == "tools/list": emit(request_id, {"tools": TOOLS})
            elif method == "tools/call":
                params = request.get("params", {})
                try: emit(request_id, tool_result(dispatch(params.get("name", ""), params.get("arguments", {}))))
                except (KeyError, TypeError, ValueError) as error: emit(request_id, tool_result({"error": str(error)}, True))
            elif method == "ping": emit(request_id, {})
            elif request_id is not None: print(json.dumps({"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": f"method not found: {method}"}}), flush=True)
        except json.JSONDecodeError: continue


if __name__ == "__main__":
    main()
