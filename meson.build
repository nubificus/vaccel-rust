project('vaccel-rust', 'c', 'rust',
  meson_version : '>=1.1',
  license : 'Apache-2.0',
  license_files : 'LICENSE',
  version : run_command('scripts/generate-version.sh',
    '', '--no-dirty', check: false).stdout().strip())

cargo_toml = files('Cargo.toml')

cargo_bin = find_program('cargo', version : '>=1.64.0', required : true)

cargo_meson_sh = files(meson.project_source_root() + '/scripts/cargo.sh')

libvaccel_dep = dependency('vaccel')

opt_async = get_option('async')
opt_async_stream = get_option('async-stream')
opt_client = get_option('client')
opt_agent = get_option('agent')

features = []
features_client = []
if opt_async.enabled()
  features = '--features=async'
endif
if opt_async_stream.enabled()
  features_client = '--features=async-stream'
endif

if opt_client.enabled()
  libvaccel_client = custom_target('vaccel-client',
    capture : false,
    console : true,
    build_by_default : true,
    install : false,
    input : files('Cargo.toml'),
    depfile : 'libvaccel_client.d',
    output : ['libvaccel_client.a'],
    env : {
      'RUSTFLAGS': '-C panic=abort',
      'CARGO_TARGET_DIR': join_paths(meson.current_build_dir(), 'cargo_target'),
      'PKG_CONFIG_PATH':
        join_paths(libvaccel_dep.get_variable(pkgconfig: 'libdir'), 'pkgconfig')
        + ':$PKG_CONFIG_PATH',
      },
    command : [cargo_meson_sh, cargo_bin, get_option('buildtype'),
      files('Cargo.toml'), 'libvaccel_client.a', '@OUTDIR@',
      features, features_client, '-p', 'vaccel-client'])

  cc = meson.get_compiler('c')
  m_dep = cc.find_library('m')
  rt_dep = cc.find_library('rt')
  vaccel_client_deps = [
    dependency('threads'),
    dependency('dl'),
    m_dep,
    rt_dep,
  ]
  libvaccel_client_dep = declare_dependency(
    dependencies : vaccel_client_deps,
    link_with : libvaccel_client)
endif

if opt_agent.enabled()
  vaccel_agent = custom_target('vaccel-agent',
    capture : false,
    console : true,
    build_by_default : true,
    install : true,
    install_dir : 'bin',
    input : files('Cargo.toml'),
    depfile : 'vaccel-agent.d',
    output : ['vaccel-agent'],
    env : {
      'RUSTFLAGS': '-C panic=abort',
      'CARGO_TARGET_DIR': join_paths(meson.current_build_dir(), 'cargo_target'),
      'PKG_CONFIG_PATH':
        join_paths(libvaccel_dep.get_variable(pkgconfig: 'libdir'), 'pkgconfig')
        + ':$PKG_CONFIG_PATH',
      },
    command : [cargo_meson_sh, cargo_bin, get_option('buildtype'),
      files('Cargo.toml'), 'vaccel-agent', '@OUTDIR@', features,
      '-p', 'vaccel-agent'])
endif

summary({
  'Use the async client/agent implementation': opt_async.enabled(),
  'Use the async streaming client/agent implementation': opt_async_stream.enabled(),
  'Build the client': opt_client.enabled(),
  'Build the agent': opt_agent.enabled(),
  },
  section : 'Configuration',
  bool_yn : true)

# only build binary dist/deb for vaccel-agent,
# we ship the client lib with the plugin.
if opt_agent.enabled()
  meson.add_dist_script(
    'scripts/dist.sh',
    'vaccel-agent',
    get_option('buildtype'),
    'async',
    opt_async.enabled() ? 'enabled' : 'disabled',
    'async-stream',
    opt_async_stream.enabled() ? 'enabled' : 'disabled',
    'client',
    'disabled',
    'agent',
    'enabled',
    )
endif
