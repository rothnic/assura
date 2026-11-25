"""Replay the installed Git and Codex hooks and the public snapshot recipe."""

import json
import os
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
ASSURA = Path(sys.argv[1]).resolve(strict=True)
PIPX = Path(sys.argv[2]).resolve(strict=True)
RUN = Path(sys.argv[3]).resolve()
RUN.mkdir()
ENV = {
    **os.environ,
    'PATH': f'{ASSURA.parent}{os.pathsep}{os.environ["PATH"]}',
    'ASSURA_BIN': str(ASSURA),
    'GIT_CONFIG_NOSYSTEM': '1',
    'GIT_CONFIG_GLOBAL': os.devnull,
    'GIT_TERMINAL_PROMPT': '0',
}
ENV.pop('GIT_INDEX_FILE', None)
ENV.pop('ASSURA_AGENT_SESSION_ID', None)
GUIDE = (ROOT / 'docs/upstream-contribution-workflow.md').read_text()
SNAPSHOT = GUIDE.split('```sh\n', 1)[1].split('\n)\n', 1)[0] + '\n)\n'
VERIFY = GUIDE.split('```sh\n', 2)[2].split('```', 1)[0]
FILE = 'pilot/gated_item.py'
SCRATCH = 'pilot/throwaway.txt'
CONFIG = """structure:
  ./:
    children:
      pilot/:
        files:
          match: '*.py'
          max_lines: 250
exclude: ['.git/**', '.assura/**', '.codex/**']
"""
THRESHOLD_PROBE = """agent_feedback:
  min_interval_seconds: 1
  reminder_seconds: 0
  collection:
    min_refresh_seconds: 1
  trajectory:
    integration_ref: refs/remotes/upstream/main
    signals:
      unintegrated:
        enabled: false
      patch_size:
        enabled: true
        changed_lines: 1000
        step_lines: 500
"""


def run(cwd, *args, input_text=None, expected=0, trim=True):
    result = subprocess.run(
        args, cwd=cwd, env=ENV, input=input_text, capture_output=True, text=True, timeout=30
    )
    assert result.returncode == expected, (args, result.stdout, result.stderr)
    if trim:
        return result.stdout.strip(), result.stderr.strip()
    return result.stdout, result.stderr


def git(cwd, *args, **kwargs):
    return run(cwd, 'git', *args, **kwargs)[0]


def assura(cwd, *args, **kwargs):
    return run(cwd, str(ASSURA), *args, **kwargs)[0]


def write(cwd, name, content):
    path = cwd / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def commit(cwd, title, *paths):
    git(cwd, 'add', '--', *paths)
    return run(cwd, 'git', 'commit', '--quiet', '-m', title)


def lines(label, count):
    return ''.join(f'# {label} {n}\n' for n in range(count))


def snapshot(cwd, branch='main', expected=0):
    return run(cwd, 'sh', '-c', SNAPSHOT + '\ncapture_pr_snapshot "$@"', 'snapshot',
               'upstream', branch, expected=expected)[0].splitlines()


def verify(cwd, reviewed, expected=0):
    return run(cwd, 'sh', '-c', SNAPSHOT + '\nreviewed="$1"\n' + VERIFY,
               'verify', '\n'.join(reviewed), expected=expected)


def file_check(cwd, base, expected=0):
    return json.loads(assura(cwd, 'check-change', '--base', base, '--head', 'HEAD',
                             '--format', 'json', expected=expected))


def new_project(name):
    project = RUN / name
    project.mkdir()
    git(project, 'init', '--quiet', '-b', 'main')
    git(project, 'config', 'user.name', 'Assura replay')
    git(project, 'config', 'user.email', 'assura-replay@example.invalid')
    return project


def hook(project, stage, event, command, path=None):
    hooks = json.loads((project / '.codex/hooks.json').read_text())['hooks']
    entry = hooks[event][0]['hooks'][0]['command']
    payload = {'cwd': str(project), 'session_id': project.name, 'hook_event_name': event,
               'tool_name': 'Bash', 'tool_input': {'command': command}}
    if path:
        payload['tool_input']['file_path'] = path
    output, _ = run(project, 'sh', '-c', entry, input_text=json.dumps(payload))
    specific = json.loads(output)['hookSpecificOutput'] if output else {}
    context = specific.get('additionalContext', '')
    if context:
        assert specific['hookEventName'] == event
    assert len(context.encode()) <= 256
    return {'stage': stage, 'event': event, 'context': context, 'bytes': len(context.encode())}


def replay(name, options='', exercise_preflight=False):
    project = new_project(name)
    config = CONFIG + options
    write(project, '.assura/config.yml', config)
    write(project, '.git/info/exclude', '.assura/\n.codex/\n')
    retained = lines('retained', 54)
    write(project, FILE, retained + lines('legacy', 1575))
    write(project, SCRATCH, 'original\n')
    for path in PR_PATHS:
        if path.startswith(('src/', 'tests/')):
            write(project, path, git(PIPX, 'show', f'{PR_BASE}:{path}', trim=False))
    git(project, 'add', '-f', '.assura/config.yml')
    commit(project, 'base', FILE, SCRATCH, 'src', 'tests')
    base = git(project, 'rev-parse', 'HEAD')
    upstream = RUN / f'{name}-upstream'
    git(project, 'clone', '--quiet', '--bare', '.', str(upstream))
    git(project, 'remote', 'add', 'upstream', str(upstream))
    git(project, 'fetch', '--quiet', 'upstream')
    git(project, 'switch', '--quiet', '-c', 'development')
    assura(project, 'hooks', 'install', '.')
    assura(project, 'agent', 'integration', 'install', 'codex', '.')
    assura(project, 'agent', 'integration', 'activate', 'codex', '.')
    assert git(project, 'status', '--porcelain') == ''
    events = [hook(project, 'start', 'SessionStart', ''),
              hook(project, 'prompt', 'UserPromptSubmit', '')]
    commits = []

    def checkpoint(stage, changes):
        for path, content in changes.items():
            events.append(hook(project, stage, 'PreToolUse', 'edit', path))
            write(project, path, content)
        if options:
            time.sleep(1.1)
            assura(project, 'agent', 'nudge', '--delivery', 'inspect')
        for path in changes:
            events.append(hook(project, stage, 'PostToolUse', 'edit', path))
        stdout, stderr = commit(project, stage, *changes)
        commits.append({'stage': stage, 'exit': 0, 'warning': 'allows commits with warnings' in stdout + stderr})
        events.append(hook(project, stage, 'PostToolUse', 'git commit'))

    checkpoint('rewrite', {FILE: retained + lines('replacement', 286)})
    shrunk = git(project, 'diff', '--numstat', base, 'HEAD')
    assert shrunk == f'286\t1575\t{FILE}'
    assert len(git(project, 'show', f'HEAD:{FILE}').splitlines()) == 340
    assert file_check(project, base)['success']
    checkpoint('temporary-edit', {SCRATCH: 'original\n' + lines('temporary', 900)})
    checkpoint('revert-edit', {SCRATCH: 'original\n'})
    assert git(project, 'diff', '--numstat', base, 'HEAD') == shrunk
    checkpoint('true-growth', {FILE: retained + lines('replacement', 286) + lines('growth', 1400)})
    finding = file_check(project, base, expected=1)['findings'][0]
    assert (finding['actual'], finding['baseline_lines'], finding['overage']) == (1740, 1629, 111)
    assert commits[-1]['warning']
    git(project, 'config', 'assura.stagedPreCommit', 'true')
    pre_attempt = git(project, 'rev-parse', 'HEAD')
    write(project, FILE, (project / FILE).read_text() + lines('extra', 1))
    git(project, 'add', FILE)
    stdout, stderr = run(project, 'git', 'commit', '--quiet', '-m', 'strict growth', expected=1)
    assert '1741 lines' in stdout + stderr and git(project, 'rev-parse', 'HEAD') == pre_attempt
    git(project, 'restore', '--source=HEAD', '--staged', '--worktree', FILE)
    git(project, 'config', '--unset', 'assura.stagedPreCommit')
    checkpoint('published-patch', {path: git(PIPX, 'show', f'{PR_HEAD}:{path}', trim=False) for path in PR_PATHS})
    assert git(project, 'diff', '--no-ext-diff', 'HEAD^', 'HEAD') == PR_DIFF
    evidence = {'commits': commits, 'explicit_enforcement_exit': 1, 'events': events,
                'review_problems': []}
    if exercise_preflight:
        evidence['review_problems'] = boundary_controls(project, upstream, base)
    return evidence


def boundary_controls(project, upstream, base):
    head = git(project, 'rev-parse', 'HEAD')
    git(project, 'remote', 'add', 'fork', str(upstream))
    git(project, 'update-ref', 'refs/remotes/fork/development', head)
    git(project, 'branch', '--set-upstream-to=fork/development')
    assert git(project, 'rev-parse', '@{upstream}') == head
    original = snapshot(project)
    assert original == ['upstream', 'main', head, base, base]
    verify(project, original)
    snapshot(project, 'missing', expected=1)
    shallow = RUN / 'shallow'
    git(project, 'clone', '--quiet', '--depth=1', '--origin=upstream', upstream.as_uri(), str(shallow))
    snapshot(shallow, expected=1)
    tree = git(project, 'rev-parse', 'HEAD^{tree}')
    advanced = git(project, 'commit-tree', tree, '-p', base, '-m', 'advance target')
    git(upstream, 'fetch', '--quiet', '--no-write-fetch-head', str(project), advanced)
    git(upstream, 'update-ref', 'refs/heads/main', advanced)
    new_target = snapshot(project)
    assert new_target == ['upstream', 'main', head, advanced, base]
    assert 'Refresh evidence' in verify(project, original, expected=1)[1]
    advanced_head = git(project, 'commit-tree', tree, '-p', head, '-m', 'advance head')
    git(project, 'update-ref', 'HEAD', advanced_head)
    assert snapshot(project) == ['upstream', 'main', advanced_head, advanced, base]
    assert 'Refresh evidence' in verify(project, new_target, expected=1)[1]
    write(project, 'untracked.txt', 'worktree only\n')
    assert git(project, 'status', '--porcelain') == '?? untracked.txt'
    snapshot(project, expected=1)
    (project / 'untracked.txt').unlink()
    (project / 'data.bin').write_bytes(b'\0\1\xff')
    commit(project, 'binary coverage', 'data.bin')
    assert git(project, 'diff', '--numstat', advanced_head, 'HEAD') == '-\t-\tdata.bin'
    other = git(project, 'commit-tree', tree, '-p', base, '-m', 'other branch')
    merged_a = git(project, 'commit-tree', tree, '-p', advanced, '-p', other, '-m', 'merge A')
    merged_b = git(project, 'commit-tree', tree, '-p', other, '-p', advanced, '-m', 'merge B')
    git(upstream, 'fetch', '--quiet', '--no-write-fetch-head', str(project), merged_a)
    git(upstream, 'update-ref', 'refs/heads/main', merged_a)
    git(project, 'update-ref', 'HEAD', merged_b)
    snapshot(project, expected=1)
    return ['push tracking differs from review target', 'missing target', 'shallow history',
            'stale target', 'stale head', 'untracked path', 'binary line count unknown',
            'ambiguous merge base']


assert ASSURA.name == 'assura'
assert run(ROOT, 'sh', '-c', 'command -v assura')[0] == str(ASSURA)
PR_HEAD = '78eaea56aadea0d786680246024396c12b6982a8'
PR_BASE = 'dbb0e420a831220edabf53819b7a1b3870de0d52'
PR_PATHS = git(PIPX, 'diff', '--name-only', PR_BASE, PR_HEAD).splitlines()
PR_NUMSTAT = git(PIPX, 'diff', '--numstat', PR_BASE, PR_HEAD)
PR_DIFF = git(PIPX, 'diff', '--no-ext-diff', PR_BASE, PR_HEAD)
assert PR_NUMSTAT == ('1\t0\tchangelog.d/+run-local-dir-infer-app.bugfix.rst\n'
                      '28\t8\tsrc/pipx/commands/run.py\n33\t0\ttests/test_run.py')
results = {
    'current': replay('current'),
    'proposed': replay('proposed', exercise_preflight=True),
    'threshold_probe': replay('threshold-probe', options=THRESHOLD_PROBE),
}
for name in ('current', 'proposed'):
    assert all(event['bytes'] == 0 for event in results[name]['events']
               if event['event'] == 'PostToolUse' and event['stage'] == 'revert-edit')
    assert all('[trajectory_stats]' not in event['context'] for event in results[name]['events'])
assert any('[trajectory_stats]' in event['context'] for event in results['threshold_probe']['events']
           if event['event'] == 'PostToolUse' and event['stage'] == 'revert-edit')
for name, evidence in results.items():
    sizes = [event['bytes'] for event in evidence['events'] if event['bytes']]
    print(f'{name}: {len(evidence["commits"])} commits, {len(sizes)} context messages, '
          f'{sum(sizes)} UTF-8 bytes, {len(evidence["review_problems"])} boundary controls')
(RUN / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
