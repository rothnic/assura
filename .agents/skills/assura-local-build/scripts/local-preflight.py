#!/usr/bin/env python3
"""Check explicit local validation inputs, then exec the unchanged command."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile


SOCKET_SUFFIX = '.tmpXXXXXX/hot-session-external-config-project.sock'
NODE_IDENTITY = """const fs=require('node:fs'),c=require('node:crypto');
console.log(JSON.stringify({cwd:process.cwd(),path:process.env.PATH,
exec_path:process.execPath,version:process.version,
sha256:c.createHash('sha256').update(fs.readFileSync(process.execPath)).digest('hex')}));"""


def identity(filename):
    path = Path(filename).resolve(strict=True)
    return {'path': str(filename), 'canonical': str(path),
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def probe(argv, cwd, record):
    result = subprocess.run(argv, cwd=cwd, capture_output=True, text=True, timeout=15)
    record.setdefault('probes', []).append({
        'argv': [str(value) for value in argv], 'cwd': str(cwd),
        'exit': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr,
    })
    if result.returncode:
        raise ValueError(f'identity probe exited {result.returncode}: {argv}')
    return result.stdout.strip()


def cargo_target(repo, command, requested, record):
    cargo = command[0] if Path(command[0]).name == 'cargo' else shutil.which('cargo')
    if not cargo:
        raise ValueError('cargo is required to resolve the effective target')
    metadata_args = [cargo]
    manifest_args = []
    options = command[1:] if Path(command[0]).name == 'cargo' else []
    if options and options[0].startswith('+'):
        metadata_args.append(options[0])
    target_override = None
    for index, option in enumerate(options):
        if option == '--config':
            metadata_args.extend([option, options[index + 1]])
        elif option.startswith('--config='):
            metadata_args.append(option)
        elif option == '--manifest-path':
            manifest_args.extend([option, options[index + 1]])
        elif option.startswith('--manifest-path='):
            manifest_args.append(option)
        elif option == '--target-dir':
            target_override = options[index + 1]
        elif option.startswith('--target-dir='):
            target_override = option.split('=', 1)[1]
    metadata_args.extend(['metadata', '--offline', '--locked', '--no-deps', '--format-version', '1'])
    metadata_args.extend(manifest_args)
    metadata = json.loads(probe(metadata_args, repo, record))
    if Path(metadata['workspace_root']).resolve() != repo:
        raise ValueError('Cargo workspace differs from the selected repository')
    raw = target_override or metadata['target_directory']
    target = (repo / raw).resolve()
    record['target'] = {'effective': raw, 'canonical': str(target), 'requested': requested}
    if requested is not None and (repo / requested).resolve() != target:
        raise ValueError('--target-dir differs from the effective Cargo target; inputs were not changed')
    for parent in (target, *target.parents):
        manifest = parent / 'Cargo.toml'
        if (parent / '.git').exists() and manifest.is_file():
            if re.search(r'^name\s*=\s*"assura"\s*$', manifest.read_text(), re.MULTILINE) and parent != repo:
                raise ValueError(f'target resolves inside another Assura checkout: {parent}')
    if not target.is_relative_to(repo):
        if requested is None:
            raise ValueError('external target requires --target-dir as explicit task-dedicated selection')
        if not target.is_dir() or target.stat().st_uid != os.getuid() or target.stat().st_mode & 0o022:
            raise ValueError('external target must exist, be caller-owned and not writable by others')
        record['target']['dedication'] = 'caller-declared task directory; ownership and mode checked'
    # These routes reach website_demo, which reads checkout-local target/debug/assura-full.
    if 'xtask' in command:
        mode = command[command.index('xtask') + 1:command.index('xtask') + 2]
        if mode and mode[0] in ('docs', 'pr', 'full', 'website-demo-data', 'website-config-examples'):
            if target != (repo / 'target').resolve():
                raise ValueError('this xtask website-demo route requires the checkout-local target')
    return target


def socket_tmpdir(selected, suffix, record):
    path = Path(selected)
    if not path.is_absolute() or not path.is_dir():
        raise ValueError('socket TMPDIR must be an existing absolute task directory')
    info = path.stat()
    record['temporary'] = {'path': str(path), 'canonical': str(path.resolve()),
                           'device': info.st_dev, 'mode': format(stat.S_IMODE(info.st_mode), '04o')}
    if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise ValueError('socket TMPDIR must be caller-owned with mode 0700')
    disk = probe(['df', '-P', str(path)], path, record)
    source = disk.splitlines()[-1].split()[0]
    record['temporary']['filesystem_source'] = source
    if not source.startswith('/dev/') or source.startswith(('/dev/ram', '/dev/zram')):
        raise ValueError(f'TMPDIR disk backing is unproven ({source}); select a verified disk-backed directory')
    relative = Path(suffix)
    if relative.is_absolute() or '..' in relative.parts:
        raise ValueError('socket suffix must stay under TMPDIR')
    projected = str(path / relative)
    size = len(os.fsencode(projected)) + 1
    record['temporary'].update(socket_path=projected, socket_bytes_with_nul=size,
                               darwin_capacity=104 if sys.platform == 'darwin' else None)
    if sys.platform == 'darwin' and size > 104:
        raise ValueError(f'projected Unix socket needs {size} bytes including NUL; Darwin capacity is 104')


def bash_mapfile(repo, record):
    bash = shutil.which('bash')
    if not bash:
        raise ValueError('bash was not found in the inherited PATH')
    record['bash'] = identity(bash)
    output = probe([bash, '-c', 'printf "%s\\n" "$BASH" "$BASH_VERSION"; type -t mapfile'], repo, record)
    lines = output.splitlines()
    if len(lines) != 3 or lines[2] != 'builtin':
        raise ValueError('selected Bash does not provide the mapfile builtin')
    record['bash'].update(runtime=identity(lines[0]), version=lines[1], mapfile=lines[2])


def website_tools(repo, record):
    website = repo / 'website'
    package = website / 'package.json'
    data = json.loads(package.read_text())
    selected = next((name for name in ('pnpm', 'npm', 'node') if shutil.which(name)), None)
    if selected is None:
        raise ValueError('website tools profile found no pnpm, npm or node')
    tools = {'selected': selected, 'command': identity(shutil.which(selected)),
             'package': identity(package), 'lock': identity(website / 'pnpm-lock.yaml'),
             'proof': 'no-build identity probe; not arbitrary nested-build evidence'}
    record['website'] = tools
    if data.get('scripts', {}).get('build') not in ('astro build', 'ASTRO_TELEMETRY_DISABLED=1 astro build'):
        raise ValueError('website build script changed; the simple Astro identity probe no longer covers its route')
    version = probe([shutil.which(selected), '--version'], website, record)
    tools['version'] = version
    pin = data.get('packageManager', '')
    if selected == 'pnpm' and pin.startswith('pnpm@') and version != pin.split('@', 1)[1]:
        raise ValueError(f'selected pnpm {version} differs from packageManager {pin}')
    if selected == 'pnpm':
        argv = [shutil.which(selected), '--dir', 'website', 'exec', 'node', '-e', NODE_IDENTITY]
        cwd = repo
    elif selected == 'npm':
        argv = [shutil.which(selected), 'exec', '--offline', '--yes=false', '--', 'node', '-e', NODE_IDENTITY]
        cwd = website
    else:
        argv = [shutil.which(selected), '-e', NODE_IDENTITY]
        cwd = website
    node = json.loads(probe(argv, cwd, record))
    tools['child'] = node
    if Path(node['cwd']).resolve() != website.resolve():
        raise ValueError('website identity probe ran outside the selected website directory')
    actual = identity(node['exec_path'])
    if actual['sha256'] != node['sha256']:
        raise ValueError('website child Node executable hash does not match the observed file')
    tools['node'] = actual


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True)
    parser.add_argument('--target-dir', help='assert effective target; explicitly declare external task ownership')
    parser.add_argument('--tmpdir', help='assert inherited TMPDIR; does not set it')
    parser.add_argument('--socket-relative', nargs='?', const=SOCKET_SUFFIX)
    parser.add_argument('--require-bash-mapfile', action='store_true')
    parser.add_argument('--website-tools', action='store_true')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('supply the original command after --')
    repo = Path(args.repo).resolve()
    record = {'ok': False, 'command_started': False, 'argv': command, 'cwd': str(Path.cwd().resolve()),
              'environment': {name: os.environ.get(name) for name in (
                  'PATH', 'CARGO_TARGET_DIR', 'TMPDIR', 'CARGO_BUILD_JOBS', 'RUST_TEST_THREADS')}}
    try:
        if Path.cwd().resolve() != repo:
            raise ValueError('caller cwd differs from --repo; no directory change was made')
        top = probe(['git', 'rev-parse', '--show-toplevel'], repo, record)
        if Path(top).resolve() != repo:
            raise ValueError('--repo is not the Git checkout root')
        record['source'] = {'root': top, 'head': probe(['git', 'rev-parse', 'HEAD'], repo, record),
                            'tree': probe(['git', 'rev-parse', 'HEAD^{tree}'], repo, record),
                            'dirty': probe(['git', 'status', '--porcelain=v1'], repo, record)}
        cargo_target(repo, command, args.target_dir, record)
        selected_tmp = os.environ.get('TMPDIR') or tempfile.gettempdir()
        if args.tmpdir and Path(args.tmpdir).resolve() != Path(selected_tmp).resolve():
            raise ValueError('--tmpdir differs from inherited TMPDIR; inputs were not changed')
        if args.socket_relative:
            socket_tmpdir(selected_tmp, args.socket_relative, record)
        if args.require_bash_mapfile:
            bash_mapfile(repo, record)
        if args.website_tools:
            website_tools(repo, record)
        record.update(ok=True, command_started=True, launch='exec; caller captures the command exit')
    except (ValueError, OSError, KeyError, IndexError, subprocess.SubprocessError) as error:
        record['error'] = str(error)
        print(json.dumps(record), file=sys.stderr, flush=True)
        return 2
    print(json.dumps(record), file=sys.stderr, flush=True)
    try:
        os.execvpe(command[0], command, os.environ)
    except OSError as error:
        print(f'command launch failed: {error}', file=sys.stderr)
        return 127 if isinstance(error, FileNotFoundError) else 126


if __name__ == '__main__':
    sys.exit(main())
