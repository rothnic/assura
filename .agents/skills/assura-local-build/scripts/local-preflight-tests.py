#!/usr/bin/env python3
"""Fake-tool controls: no Cargo compilation, website build or dependency install."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


HELPER = Path(__file__).with_name('local-preflight.py')
SPEC = importlib.util.spec_from_file_location('local_preflight', HELPER)
PREFLIGHT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PREFLIGHT)
CARGO = '''import json,os,pathlib,sys
if 'metadata' in sys.argv:
 p=pathlib.Path(os.environ['METADATA_LOG'])
 p.write_text(json.dumps(sys.argv[1:]))
 print(json.dumps({'workspace_root':os.environ['REPO'],
 'target_directory':os.environ.get('RESOLVED_TARGET',os.environ['REPO']+'/target')}))
else:
 pathlib.Path(os.environ['COMMAND_LOG']).write_text(json.dumps({
 'argv':sys.argv[1:],'cwd':os.getcwd(),'env':dict(os.environ)}))
 print('original command stdout')
 sys.exit(37)
'''
WEBSITE_TOOL = '''import hashlib,json,os,pathlib,sys
if sys.argv[1:]==['--version']:
 print(os.environ.get('TOOL_VERSION',{'pnpm':'10.25.0','npm':'10.0.0','node':'v24.19.0'}[pathlib.Path(sys.argv[0]).name]))
else:
 node=pathlib.Path(os.environ['NODE_FILE'])
 print(json.dumps({'cwd':os.environ.get('PROBE_CWD',os.environ['REPO']+'/website'),
 'path':os.environ['REPO']+'/website/node_modules/.bin:'+os.environ['PATH'],
 'exec_path':str(node),'version':'v24.19.0',
 'sha256':os.environ.get('NODE_HASH',hashlib.sha256(node.read_bytes()).hexdigest())}))
'''


def executable(path, content):
    path.write_text('#!' + sys.executable + '\n' + content)
    path.chmod(0o700)


class LocalPreflightControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(
            prefix='dx01-control-', dir=os.environ.get('DX01_CONTROL_ROOT'))
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / 'candidate'
        self.repo.mkdir()
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.command_log = self.root / 'command.json'
        self.metadata_log = self.root / 'metadata.json'
        (self.bin / 'git').symlink_to('/usr/bin/git')
        (self.bin / 'df').symlink_to('/bin/df')
        self.env = {'PATH': str(self.bin), 'HOME': str(self.root),
                    'REPO': str(self.repo), 'METADATA_LOG': str(self.metadata_log),
                    'COMMAND_LOG': str(self.command_log), 'UNCHANGED_INPUT': "spaces and 'quotes'",
                    'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull}
        self.git('init', '-q')
        self.git('config', 'core.hooksPath', str(self.root / 'no-hooks'))
        (self.repo / 'Cargo.toml').write_text('[package]\nname = "assura"\nversion = "0.1.0"\n')
        (self.repo / 'source.txt').write_text('preserve these bytes\n')
        website = self.repo / 'website'
        website.mkdir()
        (website / 'package.json').write_text(json.dumps({
            'packageManager': 'pnpm@10.25.0',
            'scripts': {'build': 'ASTRO_TELEMETRY_DISABLED=1 astro build'}}))
        (website / 'pnpm-lock.yaml').write_text("lockfileVersion: '9.0'\n")
        self.git('add', '.')
        self.git('-c', 'user.name=Control', '-c', 'user.email=control@example.invalid',
                 '-c', 'commit.gpgSign=false', 'commit', '-qm', 'fixture')
        executable(self.bin / 'cargo', CARGO)

    def git(self, *args):
        return subprocess.run(['/usr/bin/git', *args], cwd=self.repo, env=self.env,
                              check=True, capture_output=True, text=True).stdout.strip()

    def invoke(self, options=(), command=('cargo', 'test', 'space argument', "a'b"), repo=None):
        before = {name: (self.repo / name).read_bytes() for name in self.git('ls-files').splitlines()}
        result = subprocess.run([sys.executable, str(HELPER), '--repo', str(repo or self.repo),
                                 *options, '--', *command], cwd=self.repo, env=self.env,
                                capture_output=True, text=True, timeout=25)
        record = json.loads(result.stderr.splitlines()[0])
        self.assertEqual(before, {name: (self.repo / name).read_bytes() for name in before})
        if os.environ.get('DX01_CONTROL_RECEIPTS'):
            with Path(os.environ['DX01_CONTROL_RECEIPTS']).open('a') as output:
                output.write(json.dumps({'control': self.id(), 'argv': result.args,
                    'exit': result.returncode, 'stdout': result.stdout, 'preflight': record,
                    'metadata_probe_invoked': self.metadata_log.exists(),
                    'supplied_command_invoked': self.command_log.exists()}) + '\n')
        return result, record

    def rejected(self, options=(), text='', **kwargs):
        result, record = self.invoke(options, **kwargs)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertFalse(record['command_started'])
        self.assertFalse(self.command_log.exists())
        self.assertIn(text, record['error'])
        return record

    def test_local_target_preserves_command_exit_argv_cwd_environment(self):
        result, record = self.invoke()
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertTrue(record['ok'])
        self.assertEqual(result.stdout, 'original command stdout\n')
        actual = json.loads(self.command_log.read_text())
        self.assertEqual(actual['argv'], ['test', 'space argument', "a'b"])
        self.assertEqual(actual['cwd'], str(self.repo))
        for key, value in self.env.items():
            self.assertEqual(actual['env'][key], value)
        self.assertEqual(record['target']['canonical'], str(self.repo / 'target'))
        self.assertEqual(json.loads(self.metadata_log.read_text()),
                         ['metadata', '--offline', '--locked', '--no-deps', '--format-version', '1'])

    def test_relative_target_and_cli_override_are_resolved_at_candidate(self):
        self.env['CARGO_TARGET_DIR'] = 'target/custom'
        self.env['RESOLVED_TARGET'] = str(self.repo / 'target/custom')
        result, record = self.invoke(['--target-dir', 'target/custom'])
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertEqual(record['target']['canonical'], str(self.repo / 'target/custom'))
        self.command_log.unlink()
        result, record = self.invoke(['--target-dir', 'target/cli'],
                                     command=('cargo', 'build', '--target-dir', 'target/cli'))
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertEqual(record['target']['canonical'], str(self.repo / 'target/cli'))

    def test_explicit_owned_external_target_is_permitted_for_general_cargo(self):
        target = self.root / 'dedicated-target'
        target.mkdir(mode=0o700)
        self.env['RESOLVED_TARGET'] = str(target)
        self.env['CARGO_TARGET_DIR'] = str(target)
        result, record = self.invoke(['--target-dir', str(target)], command=('cargo', 'build'))
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertIn('caller-declared', record['target']['dedication'])

    def test_external_target_requires_explicit_selection(self):
        target = self.root / 'external'
        target.mkdir()
        self.env['RESOLVED_TARGET'] = str(target)
        self.rejected(text='explicit task-dedicated')
        self.assertTrue(self.metadata_log.exists())

    def test_target_inside_another_assura_checkout_rejects_after_metadata(self):
        other = self.root / 'other'
        other.mkdir()
        (other / '.git').mkdir()
        (other / 'Cargo.toml').write_text('[package]\nname = "assura"\n')
        self.env['RESOLVED_TARGET'] = str(other / 'target')
        self.rejected(['--target-dir', str(other / 'target')], text='another Assura checkout')
        self.assertTrue(self.metadata_log.exists())
        self.assertFalse((other / 'target').exists())

    def test_expected_target_mismatch_rejects_without_environment_rewrite(self):
        self.rejected(['--target-dir', 'different-target'], text='differs from the effective')
        self.assertNotIn('CARGO_TARGET_DIR', self.env)

    def test_transitive_website_demo_routes_require_local_target(self):
        for route in ('docs', 'pr', 'full', 'website-demo-data', 'website-config-examples'):
            with self.subTest(route=route):
                target = self.root / 'dedicated-target'
                target.mkdir(mode=0o700, exist_ok=True)
                self.env['RESOLVED_TARGET'] = str(target)
                self.rejected(['--target-dir', str(target)],
                              command=('cargo', 'xtask', route), text='checkout-local target')

    def test_wrong_root_rejects_before_metadata_or_final_command(self):
        self.rejected(repo=self.root, text='caller cwd differs')
        self.assertFalse(self.metadata_log.exists())

    def test_dirty_source_and_changed_head_are_recorded_without_reset(self):
        before = self.git('rev-parse', 'HEAD')
        file = self.repo / 'source.txt'
        file.write_text('intentional source edit\n')
        result, record = self.invoke()
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertIn('source.txt', record['source']['dirty'])
        self.assertEqual(file.read_text(), 'intentional source edit\n')
        self.command_log.unlink()
        self.git('add', 'source.txt')
        self.git('-c', 'user.name=Control', '-c', 'user.email=control@example.invalid',
                 '-c', 'commit.gpgSign=false', 'commit', '-qm', 'changed')
        result, record = self.invoke()
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertNotEqual(record['source']['head'], before)
        self.assertEqual(record['source']['head'], self.git('rev-parse', 'HEAD'))
        self.assertEqual(record['source']['tree'], self.git('rev-parse', 'HEAD^{tree}'))

    def test_private_disk_tmp_and_darwin_socket_boundary(self):
        with tempfile.TemporaryDirectory(prefix='dx01.', dir='/private/var/tmp' if sys.platform == 'darwin' else '/var/tmp') as temp:
            path = Path(temp)
            suffix = 'x' * (103 - len(os.fsencode(str(path))) - 1)
            with patch.object(PREFLIGHT.sys, 'platform', 'darwin'):
                record = {}
                PREFLIGHT.socket_tmpdir(temp, suffix, record)
                self.assertEqual(record['temporary']['socket_bytes_with_nul'], 104)
                with self.assertRaisesRegex(ValueError, '105 bytes'):
                    PREFLIGHT.socket_tmpdir(temp, suffix + 'x', {})
            with patch.object(PREFLIGHT.sys, 'platform', 'linux'):
                record = {}
                PREFLIGHT.socket_tmpdir(temp, suffix + 'x', record)
                self.assertIsNone(record['temporary']['darwin_capacity'])

    def test_socket_tmp_rejects_nonprivate_or_unproven_disk_before_command(self):
        self.env['TMPDIR'] = str(self.root)
        self.root.chmod(0o755)
        self.rejected(['--socket-relative', 'socket.sock'], text='mode 0700')
        self.root.chmod(0o700)
        with patch.object(PREFLIGHT, 'probe', return_value='Filesystem blocks used avail capacity mount\ntmpfs 1 0 1 0% /tmp'):
            with self.assertRaisesRegex(ValueError, 'disk backing is unproven'):
                PREFLIGHT.socket_tmpdir(str(self.root), 'socket.sock', {})

    @unittest.skipUnless(sys.platform == 'darwin', 'Darwin socket rejection is platform-specific')
    def test_long_socket_tmp_rejects_without_changing_caller_bytes(self):
        deep = self.root / ('d' * 90)
        deep.mkdir(mode=0o700)
        marker = deep / 'caller-file'
        marker.write_text('keep')
        self.env['TMPDIR'] = str(deep)
        self.rejected(['--socket-relative'], text='Darwin capacity')
        self.assertEqual(marker.read_text(), 'keep')

    def test_bash_profile_uses_selected_tool_and_is_optional(self):
        executable(self.bin / 'bash', 'print("/bin/bash\\n3.2.57\\nmissing")\n')
        self.rejected(['--require-bash-mapfile'], text='mapfile builtin')
        result, record = self.invoke()
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertNotIn('bash', record)
        self.command_log.unlink()
        executable(self.bin / 'bash', 'print("/bin/bash\\n5.3.8\\nbuiltin")\n')
        result, record = self.invoke(['--require-bash-mapfile'])
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertEqual(record['bash']['path'], str(self.bin / 'bash'))

    def website(self, manager):
        executable(self.bin / manager, WEBSITE_TOOL)
        node = self.bin / 'node'
        if not node.exists():
            executable(node, WEBSITE_TOOL)
        self.env['NODE_FILE'] = str(node)

    def test_website_selected_fallbacks_and_probe_receipts(self):
        for manager in ('pnpm', 'npm', 'node'):
            with self.subTest(manager=manager):
                self.website(manager)
                result, record = self.invoke(['--website-tools'])
                self.assertEqual(result.returncode, 37, result.stderr)
                tools = record['website']
                self.assertEqual(tools['selected'], manager)
                self.assertEqual(tools['child']['cwd'], str(self.repo / 'website'))
                self.assertEqual(tools['node']['sha256'], hashlib.sha256((self.bin / 'node').read_bytes()).hexdigest())
                self.assertIn('not arbitrary nested-build', tools['proof'])
                self.assertTrue(any('exec' in p['argv'] for p in record['probes']) if manager != 'node' else True)
                self.command_log.unlink()
                (self.bin / manager).unlink()

    def test_wrong_pnpm_pin_rejects_before_final_command(self):
        self.website('pnpm')
        self.env['TOOL_VERSION'] = '10.99.0'
        self.rejected(['--website-tools'], text='packageManager pnpm@10.25.0')

    def test_website_child_cwd_and_identity_mismatches_reject(self):
        self.website('pnpm')
        self.env['PROBE_CWD'] = str(self.repo)
        self.rejected(['--website-tools'], text='outside the selected website')
        del self.env['PROBE_CWD']
        self.env['NODE_HASH'] = '0' * 64
        self.rejected(['--website-tools'], text='Node executable hash')

    def test_changed_build_script_rejects_without_claiming_nested_proof(self):
        self.website('pnpm')
        package = self.repo / 'website/package.json'
        document = json.loads(package.read_text())
        document['scripts']['build'] = 'PATH=/other astro build'
        package.write_text(json.dumps(document))
        self.rejected(['--website-tools'], text='build script changed')
        self.assertEqual(json.loads(package.read_text()), document)


    def test_forwarded_assura_config_does_not_change_metadata_arguments(self):
        command = ('cargo', 'run', '--bin', 'assura', '--', 'check',
                   '--config', '.assura/config.yml', '.')
        result, record = self.invoke(command=command)
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertEqual(json.loads(self.metadata_log.read_text()),
                         ['metadata', '--offline', '--locked', '--no-deps', '--format-version', '1'])
        self.assertEqual(json.loads(self.command_log.read_text())['argv'], list(command[1:]))

    def test_forwarded_target_option_does_not_change_effective_cargo_target(self):
        command = ('cargo', 'run', '--bin', 'assura', '--', '--target-dir', 'application-value')
        result, record = self.invoke(command=command)
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertEqual(record['target']['canonical'], str(self.repo / 'target'))
        self.assertEqual(json.loads(self.command_log.read_text())['argv'], list(command[1:]))

    def test_unsupported_source_routing_rejects_before_metadata_and_command(self):
        other = self.root / 'other-source'
        other.mkdir()
        manifest = other / 'Cargo.toml'
        manifest.write_text('[package]\nname = "other"\n')
        for command in [('cargo', '+nightly', '-Z', 'unstable-options', '-C', str(other), 'build'),
                        ('cargo', 'build', '--manifest-path', str(manifest)),
                        ('cargo', 'build', '--manifest-path=' + str(manifest))]:
            with self.subTest(command=command):
                try:
                    self.rejected(command=command, text='source routing')
                    self.assertFalse(self.metadata_log.exists())
                finally:
                    self.command_log.unlink(missing_ok=True)
                    self.metadata_log.unlink(missing_ok=True)

    def test_non_cargo_wrapper_route_is_rejected_without_unrelated_metadata(self):
        executable(self.bin / 'xtask', CARGO)
        self.rejected(command=('xtask', 'docs'), text='direct Cargo')
        self.assertFalse(self.metadata_log.exists())

    def test_xtask_alias_expansions_preserve_local_target_constraint(self):
        target = self.root / 'dedicated-target'
        target.mkdir(mode=0o700)
        self.env['RESOLVED_TARGET'] = str(target)
        forms = [('--package', 'xtask'), ('-p', 'xtask'), ('--package=xtask',), ('-pxtask',),
                 ('--manifest-path', 'xtask/Cargo.toml')]
        for options in forms:
            with self.subTest(options=options):
                try:
                    self.rejected(['--target-dir', str(target)],
                                  command=('cargo', 'run', *options, '--', 'docs'),
                                  text='checkout-local target')
                finally:
                    self.command_log.unlink(missing_ok=True)

    def test_xtask_alias_expansion_accepts_local_target_and_preserves_argv(self):
        command = ('cargo', 'run', '--package', 'xtask', '--', 'docs')
        result, record = self.invoke(command=command)
        self.assertEqual(result.returncode, 37, result.stderr)
        self.assertEqual(json.loads(self.command_log.read_text())['argv'], list(command[1:]))

    @unittest.skipUnless(sys.platform == 'darwin', 'Darwin socket rejection is platform-specific')
    def test_socket_budget_uses_literal_inherited_spelling(self):
        with tempfile.TemporaryDirectory(prefix='dx01.', dir='/private/var/tmp') as temp:
            self.env['TMPDIR'] = temp + '/.' * 45
            self.rejected(['--socket-relative'], text='Darwin capacity')

    def test_socket_profile_rejects_unset_or_empty_tmpdir_even_with_private_temp(self):
        with tempfile.TemporaryDirectory(prefix='dx01.', dir='/private/var/tmp' if sys.platform == 'darwin' else '/var/tmp') as temp:
            self.env.update(TEMP=temp, TMP=temp)
            for inherited in (None, ''):
                with self.subTest(TMPDIR=inherited):
                    if inherited is None:
                        self.env.pop('TMPDIR', None)
                    else:
                        self.env['TMPDIR'] = inherited
                    try:
                        self.rejected(['--socket-relative'], text='nonempty inherited TMPDIR')
                    finally:
                        self.command_log.unlink(missing_ok=True)


if __name__ == '__main__':
    unittest.main(verbosity=2)
