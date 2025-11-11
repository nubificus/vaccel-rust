project('vaccel-rust', 'c', 'rust',
  meson_version : '>=1.1',
  license : 'Apache-2.0',
  license_files : 'LICENSE',
  version : run_command('sh', '-c',
    'git submodule update --init >/dev/null && ' +
    'scripts/common/generate-version.sh . --no-dirty',
    check: false).stdout().strip())

cargo_bin = find_program('cargo', version : '>=1.64.0', required : true)
cargo_meson_sh = files(meson.project_source_root() + '/scripts/common/cargo.sh')
cargo_toml = files('Cargo.toml')

glob = '''
from pathlib import Path
files = filter(lambda p: p.suffix in "@1@".split(","), Path('@0@').glob("**/*"))
for f in files: print(f)
'''
glob_dir = '''
from pathlib import Path
files = filter(lambda p: p.is_dir() and p.stem.startswith("@1@"), Path("@0@").glob("*"))
for f in files: print(f)
'''
pymod = import('python')
python = pymod.find_installation()

cc = meson.get_compiler('c')
cc_print = run_command(cc.cmd_array(), '-print-search-dirs',
  check: false).stdout().strip().split('\n')
cc_lib_dirs = []
foreach l : cc_print
  if l.contains('libraries')
    cc_lib_dirs = l.split('=')[1].split(':')
  endif
endforeach
llvm_lib_dirs = []
foreach d : cc_lib_dirs
  llvm_dirs = run_command(python, '-c',
    glob_dir.format(d, 'llvm-'),
    check: true).stdout().split('\n')
  foreach ldir : llvm_dirs
    if ldir != ''
      llvm_lib_dirs += ldir / 'lib'
    endif
  endforeach
endforeach
libclang_dep = cc.find_library('clang', dirs : llvm_lib_dirs)

libvaccel_dep = dependency('vaccel', version : '>=0.7.0')
vaccel_sources = run_command(python, '-c',
  glob.format(libvaccel_dep.get_variable('includedir'), '.h'),
  check: true).stdout().strip().split('\n')

workspace_sources = ['Cargo.toml']
vaccel_bindings_sources = run_command(python, '-c',
  glob.format(join_paths(meson.current_source_dir(), 'vaccel-bindings'), '.toml,.rs,.h'),
  check: true).stdout().strip().split('\n')
vaccel_rpc_proto_sources = run_command(python, '-c',
  glob.format(join_paths(meson.current_source_dir(), 'vaccel-rpc-proto'), '.toml,.rs,.proto'),
  check: true).stdout().strip().split('\n')
common_sources = workspace_sources + vaccel_bindings_sources + vaccel_rpc_proto_sources
common_sources += vaccel_sources

opt_async_stream = get_option('async-stream')
opt_async = get_option('async').enable_if(opt_async_stream.enabled())
opt_rpc_client = get_option('rpc-client')
opt_rpc_agent = get_option('rpc-agent')

features = []
features_client = []
if opt_async.enabled()
  features = '--features=async'
endif
if opt_async_stream.enabled()
  features_client = '--features=async-stream'
endif

if opt_rpc_client.enabled()
  if meson.is_subproject()
    install_client = false
  else
    install_client = true
  endif

  vaccel_rpc_client_sources = run_command(python, '-c',
    glob.format(join_paths(meson.current_source_dir(), 'vaccel-rpc-client'), '.toml,.rs'),
    check: true).stdout().strip().split('\n')

  vaccel_rpc_client = custom_target('vaccel-rpc-client',
    capture : false,
    console : true,
    build_by_default : true,
    install : install_client,
    install_dir : [get_option('libdir'), get_option('includedir')],
    input : files(vaccel_rpc_client_sources + common_sources),
    output : ['libvaccel_rpc_client.a', 'vaccel-rpc-client.h'],
    env : {
      'RUSTFLAGS': '-C panic=abort',
      'CARGO_TARGET_DIR': join_paths(meson.current_build_dir(), 'cargo_target'),
      'PKG_CONFIG_PATH':
        join_paths(libvaccel_dep.get_variable(pkgconfig: 'libdir'), 'pkgconfig')
        + ':$PKG_CONFIG_PATH',
      },
    command : [cargo_meson_sh, cargo_bin, get_option('buildtype'),
      cargo_toml, '*vaccel*client.*', '@OUTDIR@',
      features, features_client, '-p', 'vaccel-rpc-client'])

  m_dep = cc.find_library('m')
  rt_dep = cc.find_library('rt')
  vaccel_rpc_client_deps = [
    dependency('threads'),
    dependency('dl'),
    m_dep,
    rt_dep,
  ]
  libvaccel_rpc_client = vaccel_rpc_client[0]
  vaccel_rpc_client_h = vaccel_rpc_client[1]
  libvaccel_rpc_client_dep = declare_dependency(
    sources : vaccel_rpc_client_h,
    include_directories : include_directories('.'),
    dependencies : vaccel_rpc_client_deps,
    link_with : libvaccel_rpc_client)
endif

if opt_rpc_agent.enabled()
  vaccel_rpc_agent_sources = run_command(python, '-c',
    glob.format(join_paths(meson.current_source_dir(), 'vaccel-rpc-agent'), '.toml,.rs'),
    check: true).stdout().strip().split('\n')

  vaccel_rpc_agent = custom_target('vaccel-rpc-agent',
    capture : false,
    console : true,
    build_by_default : true,
    install : true,
    install_dir : get_option('bindir'),
    input : files(vaccel_rpc_agent_sources + common_sources),
    output : ['vaccel-rpc-agent'],
    env : {
      'RUSTFLAGS': '-C panic=abort',
      'CARGO_TARGET_DIR': join_paths(meson.current_build_dir(), 'cargo_target'),
      'PKG_CONFIG_PATH':
        join_paths(libvaccel_dep.get_variable(pkgconfig: 'libdir'), 'pkgconfig')
        + ':$PKG_CONFIG_PATH',
      },
    command : [cargo_meson_sh, cargo_bin, get_option('buildtype'),
      cargo_toml, 'vaccel-rpc-agent', '@OUTDIR@', features,
      '-p', 'vaccel-rpc-agent'])
endif

summary({
  'Use the async rpc client/agent implementation': opt_async.enabled(),
  'Use the async streaming rpc client/agent implementation': opt_async_stream.enabled(),
  'Build the rpc client': opt_rpc_client.enabled(),
  'Build the rpc agent': opt_rpc_agent.enabled(),
  },
  section : 'Configuration',
  bool_yn : true)

# only build binary dist/deb for vaccel-rpc-agent,
# we ship the client lib with the plugin.
if opt_rpc_agent.enabled() and not meson.is_subproject()
  meson.add_dist_script(
    'scripts/common/dist.sh',
    'vaccel-rpc-agent',
    get_option('buildtype'),
    'async',
    opt_async.enabled() ? 'enabled' : 'disabled',
    'async-stream',
    opt_async_stream.enabled() ? 'enabled' : 'disabled',
    'rpc-client',
    'disabled',
    'rpc-agent',
    'enabled',
    )
endif

if opt_rpc_client.enabled() and not meson.is_subproject()
  meson.add_dist_script(
    'scripts/common/dist.sh',
    'vaccel-rpc-client',
    get_option('buildtype'),
    'async',
    opt_async.enabled() ? 'enabled' : 'disabled',
    'async-stream',
    opt_async_stream.enabled() ? 'enabled' : 'disabled',
    'rpc-client',
    'enabled',
    'rpc-agent',
    'disabled',
    )
endif
