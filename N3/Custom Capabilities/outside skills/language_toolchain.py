#!/opt/homebrew/bin/python3
"""A deliberately bounded local MCP server for Zed language workflows.

It exposes discovery, convention reading, and fixed validation commands. It
never accepts a shell command string, so language checks remain inspectable.
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


SERVER_NAME = "language-toolchain"
MAX_TEXT = 48_000
CONVENTION_FILES = (
    "AGENTS.md",
    "CLAUDE.md",
    "pyproject.toml",
    "package.json",
    "Cargo.toml",
    "rebar.config",
    "CMakeLists.txt",
    "Package.swift",
    ".clang-format",
    "rustfmt.toml",
)
IGNORED_SOURCE_DIRS = {".git", ".next", ".swiftpm", "DerivedData", "build", "node_modules", "target"}


def result(request_id: Any, value: dict[str, Any]) -> None:
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": value}), flush=True)


def error(request_id: Any, code: int, message: str) -> None:
    print(
        json.dumps(
            {"jsonrpc": "2.0", "id": request_id, "error": {"code": code, "message": message}}
        ),
        flush=True,
    )


def tool_text(payload: Any, is_error: bool = False) -> dict[str, Any]:
    return {
        "content": [{"type": "text", "text": json.dumps(payload, indent=2, sort_keys=True)}],
        "isError": is_error,
    }


def workspace(value: str | None) -> Path:
    root = Path(value or os.getcwd()).expanduser().resolve()
    if not root.is_dir():
        raise ValueError(f"workspace is not a directory: {root}")
    return root


def command_available(command: str) -> bool:
    return shutil.which(command) is not None


def xcrun_tool_available(tool: str) -> bool:
    completed = subprocess.run(
        ["xcrun", "--sdk", "macosx", "--find", tool],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=15,
        check=False,
    )
    return completed.returncode == 0


def metal_sources(root: Path) -> list[Path]:
    sources: list[Path] = []
    for current, directories, files in os.walk(root):
        directories[:] = [name for name in directories if name not in IGNORED_SOURCE_DIRS]
        for filename in files:
            if filename.endswith(".metal"):
                sources.append(Path(current) / filename)
                if len(sources) == 64:
                    return sorted(sources)
    return sorted(sources)


def detect_toolchains(arguments: dict[str, Any]) -> dict[str, Any]:
    root = workspace(arguments.get("workspace"))
    markers = {
        "rust": ["Cargo.toml"],
        "typescript": ["package.json", "tsconfig.json"],
        "python": ["pyproject.toml", "requirements.txt", "setup.py"],
        "erlang": ["rebar.config", "erlang.mk"],
        "mlx": ["mlx", "mlx_lm", "requirements.txt", "pyproject.toml"],
        "cpp": ["CMakeLists.txt", "compile_commands.json", "meson.build"],
        "swift": ["Package.swift", ".xcodeproj"],
        "metal": [],
    }
    executables = {
        "rust": ["cargo", "rustfmt", "clippy-driver"],
        "typescript": ["node", "npm"],
        "python": ["python3", "pytest", "ruff"],
        "erlang": ["erl", "erlc", "rebar3"],
        "mlx": ["python3"],
        "cpp": ["cmake", "clang++", "ctest"],
        "swift": ["swift", "swift-format"],
        "metal": [],
    }
    languages: dict[str, dict[str, Any]] = {}
    for language, names in markers.items():
        found_markers = [marker for marker in names if (root / marker).exists()]
        if language == "metal":
            languages[language] = {
                "markers": ["*.metal"] if metal_sources(root) else [],
                "executables": {
                    "metal": xcrun_tool_available("metal"),
                    "metallib": xcrun_tool_available("metallib"),
                },
            }
        else:
            languages[language] = {
                "markers": found_markers,
                "executables": {exe: command_available(exe) for exe in executables[language]},
            }
    return {
        "workspace": str(root),
        "languages": languages,
    }


def read_project_conventions(arguments: dict[str, Any]) -> dict[str, Any]:
    root = workspace(arguments.get("workspace"))
    content: dict[str, str] = {}
    used = 0
    for relative in CONVENTION_FILES:
        candidate = root / relative
        if not candidate.is_file():
            continue
        remaining = MAX_TEXT - used
        if remaining <= 0:
            break
        text = candidate.read_text(encoding="utf-8", errors="replace")[:remaining]
        content[relative] = text
        used += len(text)
    return {"workspace": str(root), "files": content, "truncated": used >= MAX_TEXT}


def npm_command(root: Path, operation: str) -> list[str] | None:
    package = root / "package.json"
    if not package.is_file():
        return None
    try:
        scripts = json.loads(package.read_text(encoding="utf-8")).get("scripts", {})
    except json.JSONDecodeError:
        return None
    script = {"typecheck": "typecheck", "lint": "lint", "test": "test"}.get(operation)
    return ["npm", "run", script] if script and script in scripts else None


def language_command(root: Path, language: str, operation: str) -> list[str]:
    commands: dict[str, dict[str, list[str]]] = {
        "rust": {
            "format": ["cargo", "fmt", "--check"],
            "check": ["cargo", "check"],
            "test": ["cargo", "test"],
        },
        "python": {
            "syntax": ["python3", "-m", "compileall", "-q", "."],
            "test": ["python3", "-m", "pytest", "-q"],
            "lint": ["ruff", "check", "."],
        },
        "erlang": {
            "compile": ["rebar3", "compile"],
            "test": ["rebar3", "eunit"],
        },
        "mlx": {
            "import": ["python3", "-c", "import mlx.core as mx; print(mx.default_device())"],
            "test": ["python3", "-m", "pytest", "-q"],
        },
        "cpp": {
            "build": ["cmake", "--build", "build"],
            "test": ["ctest", "--test-dir", "build", "--output-on-failure"],
        },
        "swift": {
            "build": ["swift", "build"],
            "test": ["swift", "test"],
            "format": ["swift-format", "lint", "-r", "."],
        },
    }
    if language == "typescript":
        selected = npm_command(root, operation)
        if selected:
            return selected
        if operation == "typecheck" and (root / "node_modules/.bin/tsc").exists():
            return ["node_modules/.bin/tsc", "--noEmit"]
        raise ValueError("TypeScript operation requires a matching package.json script or local tsc")
    try:
        return commands[language][operation]
    except KeyError as exc:
        raise ValueError(f"unsupported {language} operation: {operation}") from exc


def run_metal_check(root: Path, operation: str, timeout: int) -> dict[str, Any]:
    sources = metal_sources(root)
    if not sources:
        raise ValueError("Metal validation requires at least one .metal source file")
    if operation not in {"syntax", "compile", "link"}:
        raise ValueError("supported Metal operations are syntax, compile, and link")
    commands: list[list[str]] = []
    outputs: list[str] = []
    exit_code = 0
    with tempfile.TemporaryDirectory(prefix="zed-metal-") as temporary_dir:
        for index, source in enumerate(sources):
            command = ["xcrun", "--sdk", "macosx", "metal"]
            if operation == "syntax":
                command.extend(["-fsyntax-only", str(source)])
            else:
                command.extend(["-c", str(source), "-o", str(Path(temporary_dir) / f"{index}.air")])
            commands.append(command)
            try:
                completed = subprocess.run(
                    command,
                    cwd=root,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    timeout=timeout,
                    check=False,
                )
            except subprocess.TimeoutExpired as exc:
                return {
                    "workspace": str(root), "language": "metal", "operation": operation,
                    "commands": commands, "exit_code": None, "timed_out": True,
                    "output": (exc.stdout or "")[-MAX_TEXT:],
                }
            outputs.append(completed.stdout)
            if completed.returncode:
                exit_code = completed.returncode
                break
        if operation == "link" and not exit_code:
            compiled_files = [str(Path(temporary_dir) / f"{index}.air") for index in range(len(sources))]
            link_command = [
                "xcrun", "--sdk", "macosx", "metallib", *compiled_files,
                "-o", str(Path(temporary_dir) / "shader.metallib"),
            ]
            commands.append(link_command)
            try:
                linked = subprocess.run(
                    link_command,
                    cwd=root,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                    timeout=timeout,
                    check=False,
                )
            except subprocess.TimeoutExpired as exc:
                return {
                    "workspace": str(root), "language": "metal", "operation": operation,
                    "commands": commands, "exit_code": None, "timed_out": True,
                    "output": (exc.stdout or "")[-MAX_TEXT:],
                }
            outputs.append(linked.stdout)
            exit_code = linked.returncode
    combined_output = "".join(outputs)
    return {
        "workspace": str(root), "language": "metal", "operation": operation,
        "commands": commands, "exit_code": exit_code,
        "output": combined_output[-MAX_TEXT:], "truncated": len(combined_output) > MAX_TEXT,
    }


def run_language_check(arguments: dict[str, Any]) -> dict[str, Any]:
    root = workspace(arguments.get("workspace"))
    language = arguments.get("language")
    operation = arguments.get("operation")
    timeout = int(arguments.get("timeout_seconds", 300))
    if language not in {"rust", "typescript", "python", "erlang", "mlx", "cpp", "swift", "metal"}:
        raise ValueError("unsupported language")
    if not isinstance(operation, str):
        raise ValueError("operation is required")
    if timeout < 1 or timeout > 900:
        raise ValueError("timeout_seconds must be between 1 and 900")
    if language == "metal":
        return run_metal_check(root, operation, timeout)
    command = language_command(root, language, operation)
    if not command_available(command[0]) and not (root / command[0]).exists():
        raise ValueError(f"required executable is unavailable: {command[0]}")
    try:
        completed = subprocess.run(
            command,
            cwd=root,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=timeout,
            check=False,
        )
        output = completed.stdout[-MAX_TEXT:]
        return {
            "workspace": str(root),
            "language": language,
            "operation": operation,
            "command": command,
            "exit_code": completed.returncode,
            "output": output,
            "truncated": len(completed.stdout) > MAX_TEXT,
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "workspace": str(root),
            "language": language,
            "operation": operation,
            "command": command,
            "exit_code": None,
            "timed_out": True,
            "output": (exc.stdout or "")[-MAX_TEXT:],
        }


TOOLS = [
    {
        "name": "detect_toolchains",
        "description": "Discover Rust, TypeScript, Python, Erlang, MLX, C++, Swift, and Metal project markers and local toolchains.",
        "inputSchema": {"type": "object", "properties": {"workspace": {"type": "string"}}},
    },
    {
        "name": "read_project_conventions",
        "description": "Read bounded project instructions and build/style configuration before making code changes.",
        "inputSchema": {"type": "object", "properties": {"workspace": {"type": "string"}}},
    },
    {
        "name": "run_language_check",
        "description": "Run a bounded, language-specific validation command; arbitrary shell commands are not accepted.",
        "inputSchema": {
            "type": "object",
            "required": ["language", "operation"],
            "properties": {
                "workspace": {"type": "string"},
                "language": {"type": "string", "enum": ["rust", "typescript", "python", "erlang", "mlx", "cpp", "swift", "metal"]},
                "operation": {"type": "string", "enum": ["format", "check", "test", "typecheck", "lint", "syntax", "compile", "link", "build", "import"]},
                "timeout_seconds": {"type": "integer", "minimum": 1, "maximum": 900},
            },
        },
    },
]


def dispatch(method: str, arguments: dict[str, Any]) -> dict[str, Any]:
    if method == "detect_toolchains":
        return detect_toolchains(arguments)
    if method == "read_project_conventions":
        return read_project_conventions(arguments)
    if method == "run_language_check":
        return run_language_check(arguments)
    raise ValueError(f"unknown tool: {method}")


def main() -> None:
    for line in sys.stdin:
        try:
            request = json.loads(line)
            method = request.get("method")
            request_id = request.get("id")
            if method == "notifications/initialized":
                continue
            if method == "initialize":
                result(
                    request_id,
                    {
                        "protocolVersion": request.get("params", {}).get("protocolVersion", "2025-03-26"),
                        "capabilities": {"tools": {"listChanged": False}},
                        "serverInfo": {"name": SERVER_NAME, "version": "1.0.0"},
                    },
                )
            elif method == "tools/list":
                result(request_id, {"tools": TOOLS})
            elif method == "tools/call":
                params = request.get("params", {})
                try:
                    result(request_id, tool_text(dispatch(params.get("name", ""), params.get("arguments", {}))))
                except (TypeError, ValueError) as exc:
                    result(request_id, tool_text({"error": str(exc)}, is_error=True))
            elif method == "ping":
                result(request_id, {})
            elif request_id is not None:
                error(request_id, -32601, f"method not found: {method}")
        except json.JSONDecodeError:
            continue


if __name__ == "__main__":
    main()
