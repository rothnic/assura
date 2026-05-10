#!/usr/bin/env python3
"""Minimal YAML reader shared by Trellis helper scripts."""

from __future__ import annotations


def _unquote(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in ('"', "'"):
        return value[1:-1]
    return value


def _strip_inline_comment(value: str) -> str:
    """Strip YAML-style inline comments while preserving quoted strings."""
    in_quote: str | None = None
    for idx, ch in enumerate(value):
        if in_quote:
            if ch == in_quote:
                in_quote = None
            continue
        if ch in ('"', "'"):
            in_quote = ch
            continue
        if ch == "#" and (idx == 0 or value[idx - 1].isspace()):
            return value[:idx]
    return value


def parse_simple_yaml(content: str) -> dict:
    """Parse the small YAML subset used by .trellis/config.yaml."""
    lines = content.splitlines()
    result: dict = {}
    _parse_yaml_block(lines, 0, 0, result)
    return result


def _parse_yaml_block(
    lines: list[str], start: int, min_indent: int, target: dict
) -> int:
    i = start
    current_list: list | None = None

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if not stripped or stripped.startswith("#"):
            i += 1
            continue

        indent = len(line) - len(line.lstrip())
        if indent < min_indent:
            break

        if stripped.startswith("- "):
            if current_list is not None:
                current_list.append(_unquote(_strip_inline_comment(stripped[2:]).strip()))
            i += 1
        elif ":" in stripped:
            key, _, value = stripped.partition(":")
            key = key.strip()
            value = _unquote(_strip_inline_comment(value).strip())
            current_list = None

            if value:
                target[key] = value
                i += 1
            else:
                next_i, next_line = _next_content_line(lines, i + 1)
                if next_i >= len(lines):
                    target[key] = {}
                    i = next_i
                elif next_line.strip().startswith("- "):
                    current_list = []
                    target[key] = current_list
                    i += 1
                else:
                    next_indent = len(next_line) - len(next_line.lstrip())
                    if next_indent > indent:
                        nested: dict = {}
                        target[key] = nested
                        i = _parse_yaml_block(lines, i + 1, next_indent, nested)
                    else:
                        target[key] = {}
                        i += 1
        else:
            i += 1

    return i


def _next_content_line(lines: list[str], start: int) -> tuple[int, str]:
    i = start
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped and not stripped.startswith("#"):
            return i, lines[i]
        i += 1
    return i, ""
