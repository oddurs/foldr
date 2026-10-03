#!/usr/bin/env python3
"""Exercise the built CLI against disposable folders and isolated recovery state."""

import json
import ctypes
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile


if sys.platform == "darwin":
    native = ctypes.CDLL(None, use_errno=True)
    native.getxattr.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_uint32, ctypes.c_int]
    native.getxattr.restype = ctypes.c_ssize_t
    native.setxattr.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_uint32, ctypes.c_int]
    native.setxattr.restype = ctypes.c_int
    native.listxattr.argtypes = [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
    native.listxattr.restype = ctypes.c_ssize_t

    def native_result(result):
        if result < 0:
            error = ctypes.get_errno()
            raise OSError(error, os.strerror(error))
        return result

    def set_xattr(path, name, value):
        buffer = ctypes.create_string_buffer(value)
        native_result(native.setxattr(os.fsencode(path), os.fsencode(name), buffer, len(value), 0, 0))

    def get_xattr(path, name):
        size = native_result(native.getxattr(os.fsencode(path), os.fsencode(name), None, 0, 0, 0))
        buffer = ctypes.create_string_buffer(max(1, size))
        actual = native_result(native.getxattr(os.fsencode(path), os.fsencode(name), buffer, size, 0, 0))
        return buffer.raw[:actual]

    def list_xattrs(path):
        size = native_result(native.listxattr(os.fsencode(path), None, 0, 0))
        buffer = ctypes.create_string_buffer(max(1, size))
        actual = native_result(native.listxattr(os.fsencode(path), buffer, size, 0))
        return [os.fsdecode(name) for name in buffer.raw[:actual].split(b"\0") if name]
else:
    set_xattr = os.setxattr
    get_xattr = os.getxattr
    list_xattrs = os.listxattr


def verify(binary):
    binary = str(Path(binary).resolve())
    with tempfile.TemporaryDirectory(prefix="foldr-check-") as temporary:
        root = Path(temporary).resolve()
        state = root / "state"
        # APFS validates UTF-8 filenames; Linux permits arbitrary non-NUL bytes.
        suffix = b"/folder-\n\x1b" if sys.platform == "darwin" else b"/folder-\xff\n\x1b"
        raw_path = os.fsencode(root) + suffix
        os.mkdir(raw_path, 0o755)
        folder = Path(os.fsdecode(raw_path))
        prefix = "com.foldr." if sys.platform == "darwin" else "user.foldr."
        other = "com.example.preserved" if sys.platform == "darwin" else "user.example.preserved"
        set_xattr(folder, other, b"unrelated\0\xff")

        def run(*arguments, json_output=True, expected=0):
            command = [binary, "--state-dir", str(state)]
            if json_output:
                command.append("--json")
            result = subprocess.run(command + list(map(str, arguments)), capture_output=True)
            if result.returncode != expected:
                raise AssertionError((arguments, result.returncode, result.stdout, result.stderr))
            if expected:
                assert not result.stdout, result.stdout
                if json_output:
                    json.loads(result.stderr)
                return result
            assert not result.stderr, result.stderr
            if not json_output:
                return result.stdout
            envelope = json.loads(result.stdout)
            assert envelope["schema_version"] == 1, envelope
            return envelope["data"]

        def attributes(path):
            return {name: get_xattr(path, name) for name in list_xattrs(path)}

        before = os.stat(folder)
        initial_attributes = attributes(folder)
        snapshot = run("inspect", folder)
        assert snapshot["path"]["bytes"] == list(raw_path)
        assert "\n" not in snapshot["path"]["display"]
        assert "\x1b" not in snapshot["path"]["display"]
        assert snapshot["identity"]["inode"] == before.st_ino
        assert snapshot["mode"] == stat.S_IMODE(before.st_mode)
        run("doctor", folder)
        after = os.stat(folder)
        assert (before.st_mode, before.st_mtime_ns, before.st_ctime_ns) == (
            after.st_mode, after.st_mtime_ns, after.st_ctime_ns
        )
        assert attributes(folder) == initial_attributes
        assert not state.exists()

        text = "an inbox\nwith a terminal escape: \x1b"
        preview = run("note", "set", folder, text, "--dry-run")
        assert preview["changes"], preview
        assert attributes(folder) == initial_attributes
        assert not state.exists()
        record = run("note", "set", folder, text)
        assert record["completed"], record
        assert get_xattr(folder, prefix + "note") == text.encode()
        assert get_xattr(folder, other) == initial_attributes[other]
        assert run("note", "get", folder)["value"] == list(text.encode())
        human = run("note", "get", folder, json_output=False)
        assert b"\x1b" not in human
        assert run("undo", "show", record["id"])["id"] == record["id"]

        set_xattr(folder, prefix + "note", b"external edit")
        run("undo", "apply", record["id"], expected=4)
        assert get_xattr(folder, prefix + "note") == b"external edit"
        set_xattr(folder, prefix + "note", text.encode())
        count = len(list(state.glob("*.json")))
        run("undo", "apply", record["id"], "--dry-run")
        assert len(list(state.glob("*.json"))) == count
        undone = run("undo", "apply", record["id"])
        assert undone["completed"], undone
        assert prefix + "note" not in list_xattrs(folder)
        assert get_xattr(folder, other) == initial_attributes[other]

        binary_value = bytes([0, 255, 10, 1, 0])
        run("attr", "set", folder, "binary", binary_value.hex(), "--hex")
        assert run("attr", "get", folder, "binary")["value"] == list(binary_value)
        assert get_xattr(folder, prefix + "binary") == binary_value

        symlink = root / "link"
        symlink.symlink_to(folder)
        run("note", "set", symlink, "through a link", expected=2)
        run("note", "set", symlink, "through a link", "--follow-symlink")
        assert get_xattr(folder, prefix + "note") == b"through a link"

        original_mode = stat.S_IMODE(os.stat(folder).st_mode)
        run("permissions", "set", folder, "0700", "--dry-run")
        assert stat.S_IMODE(os.stat(folder).st_mode) == original_mode
        permission_record = run("permissions", "set", folder, "0700")
        assert stat.S_IMODE(os.stat(folder).st_mode) == 0o700
        run("undo", "apply", permission_record["id"])
        assert stat.S_IMODE(os.stat(folder).st_mode) == original_mode

        if sys.platform == "darwin":
            original_flags = os.stat(folder).st_flags
            flag_record = run("flags", "set", folder, "--hidden", "true")
            assert os.stat(folder).st_flags & stat.UF_HIDDEN
            run("undo", "apply", flag_record["id"])
            assert os.stat(folder).st_flags == original_flags
        else:
            run("flags", "set", folder, "--hidden", "true", expected=3)

        preset = root / "inbox.toml"
        run("preset", "save", folder, "--output", preset, "--fields", "note,attrs")
        assert preset.is_file()
        run("preset", "show", preset)
        targets = [root / "destination-one", root / "destination-two"]
        for target in targets:
            target.mkdir(mode=0o750)
            set_xattr(target, other, b"keep destination metadata")
        target_modes = [stat.S_IMODE(os.stat(target).st_mode) for target in targets]
        run("preset", "apply", preset, *targets, "--dry-run")
        assert all(prefix + "note" not in list_xattrs(target) for target in targets)
        run("preset", "apply", preset, *targets)
        for target, mode in zip(targets, target_modes):
            assert get_xattr(target, prefix + "note") == b"through a link"
            assert get_xattr(target, prefix + "binary") == binary_value
            assert get_xattr(target, other) == b"keep destination metadata"
            assert stat.S_IMODE(os.stat(target).st_mode) == mode
        run("diff", targets[0], targets[1])
        exported = root / "exported.toml"
        run("preset", "export", preset, "--output", exported)
        run("preset", "show", exported)

        for shell in ("bash", "zsh", "fish"):
            assert b"foldr" in run("completions", shell, json_output=False)
        assert b"foldr" in run("man", json_output=False)

        # A closed downstream pipe must not panic or emit a diagnostic.
        process = subprocess.Popen([binary, "completions", "bash"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        process.stdout.close()
        stderr = process.stderr.read()
        assert process.wait() == 0 and not stderr, stderr
        print("End-to-end folder workflows passed")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python3 scripts/verify-cli.py PATH_TO_FOLDR")
    verify(sys.argv[1])
