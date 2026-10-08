#!/usr/bin/env python3
"""Verify released workflows independently against disposable native folders."""

import importlib.util
import json
import os
from pathlib import Path
import plistlib
import stat
import subprocess
import sys
import tempfile

spec = importlib.util.spec_from_file_location("foldr_native_verify", Path(__file__).with_name("verify-cli.py"))
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


def verify(binary):
    binary = str(Path(binary).resolve())
    with tempfile.TemporaryDirectory(prefix="foldr-v020-") as temporary:
        root = Path(temporary).resolve()
        state = root / "state"
        library = root / "library"
        source = root / "source"
        target = root / "target.toml"
        source.mkdir()
        target.mkdir()
        prefix = "com.foldr." if sys.platform == "darwin" else "user.foldr."
        foreign = "com.example.preserved" if sys.platform == "darwin" else "user.example.preserved"
        native.set_xattr(target, foreign, b"foreign\0\xff")

        def run(*arguments, expected=0, json_output=True):
            command = [binary, "--state-dir", str(state), "--preset-dir", str(library)]
            if json_output:
                command.append("--json")
            result = subprocess.run(command + list(map(str, arguments)), capture_output=True)
            assert result.returncode == expected, (arguments, result.returncode, result.stdout, result.stderr)
            if result.stdout and json_output:
                envelope = json.loads(result.stdout)
                assert envelope["schema_version"] == 1
                data = envelope["data"]
            else:
                data = result.stdout
            if expected in (0, 6):
                assert not result.stderr, result.stderr
            elif json_output:
                assert json.loads(result.stderr)["schema_version"] == 1
            return data

        def observed(folder):
            info = os.stat(folder)
            attrs = {name: native.get_xattr(folder, name) for name in native.list_xattrs(folder)}
            return info.st_mode, info.st_mtime_ns, info.st_ctime_ns, getattr(info, "st_flags", None), attrs

        run("preset", "list")
        assert not library.exists() and not state.exists()
        run("note", "set", source, "Inbox")
        run("attr", "set", source, "binary", "00ff0a", "--hex")
        preset = root / "inbox.toml"
        run("preset", "save", source, "--output", preset, "--fields", "note,attrs")
        run("preset", "install", preset, "--name", "inbox")
        assert (library / "inbox.toml").is_file()
        assert stat.S_IMODE(os.stat(library / "inbox.toml").st_mode) == 0o600
        run("preset", "list")
        installed = (library / "inbox.toml").read_bytes()
        run("preset", "install", preset, "--name", "inbox", expected=4)
        assert (library / "inbox.toml").read_bytes() == installed
        run("preset", "install", preset, "--name", "../escape", expected=2)
        assert not (root / "escape.toml").exists()

        before = observed(target)
        journals = {path.name: path.read_bytes() for path in state.glob("*.json")}
        drift = run("preset", "check", "--name", "inbox", target, expected=6)
        assert drift["targets"][0]["status"] == "drift", drift
        run("preset", "apply", "--name", "inbox", target, "--dry-run")
        run("permissions", "explain", target)
        assert observed(target) == before
        assert {path.name: path.read_bytes() for path in state.glob("*.json")} == journals
        record = run("preset", "apply", "--name", "inbox", target)
        assert record["completed"]
        assert native.get_xattr(target, prefix + "note") == b"Inbox"
        assert native.get_xattr(target, prefix + "binary") == b"\0\xff\n"
        assert native.get_xattr(target, foreign) == b"foreign\0\xff"
        compliant = run("preset", "check", "--name", "inbox", target)
        assert compliant["targets"][0]["status"] == "compliant", compliant
        from_file = run("preset", "check", preset, target)
        assert from_file["targets"] == compliant["targets"]
        after = observed(target)
        run("undo", "history", "--path", target, "--limit", "1")
        assert observed(target) == after
        mixed = run("preset", "check", "--name", "inbox", target, root / "missing", expected=1)
        assert mixed["targets"][0]["status"] == "compliant"
        assert mixed["targets"][1]["status"] == "indeterminate"
        run("undo", "apply", record["id"])
        assert prefix + "note" not in native.list_xattrs(target)
        assert native.get_xattr(target, foreign) == b"foreign\0\xff"

        alias = root / "library-link"
        alias.symlink_to(library, target_is_directory=True)
        result = subprocess.run([binary, "--json", "--preset-dir", str(alias), "preset", "list"], capture_output=True)
        assert result.returncode != 0 and result.stderr

        if sys.platform == "darwin":
            key = "com.apple.metadata:_kMDItemUserTags"
            finder = "com.apple.FinderInfo"
            original = plistlib.dumps(["Keep\n6", "Café\n2"], fmt=plistlib.FMT_BINARY)
            finder_bytes = bytearray(32)
            finder_bytes[9] = 6
            native.set_xattr(target, key, original)
            native.set_xattr(target, finder, bytes(finder_bytes))
            before = observed(target)
            run("tags", "list", target)
            run("tags", "add", target, "Inbox", "--dry-run")
            assert observed(target) == before
            tag_record = run("tags", "add", target, "Inbox")
            encoded = native.get_xattr(target, key)
            tags = plistlib.loads(encoded)
            assert "Keep\n6" in tags and "Café\n2" in tags
            assert any(tag.split("\n")[0] == "Inbox" for tag in tags)
            assert native.get_xattr(target, finder) == bytes(finder_bytes)
            assert native.get_xattr(target, foreign) == b"foreign\0\xff"
            native.set_xattr(target, key, plistlib.dumps(["External\n1"], fmt=plistlib.FMT_BINARY))
            run("undo", "apply", tag_record["id"], expected=4)
            native.set_xattr(target, key, encoded)
            run("undo", "apply", tag_record["id"])
            assert native.get_xattr(target, key) == original
            assert native.get_xattr(target, finder) == bytes(finder_bytes)
        else:
            run("tags", "list", target, expected=3)
            run("tags", "add", target, "Inbox", "--dry-run", expected=3)
        print("v0.2 named profiles, checks, explanations, history and native tags passed")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python3 scripts/verify-v020.py PATH_TO_FOLDR")
    verify(sys.argv[1])
