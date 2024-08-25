project('vaccel-rust', 'c', 'rust',
  meson_version : '>=1.1',
  license : 'Apache-2.0',
  license_files : 'LICENSE',
  version : run_command('scripts/generate-version.sh',
    '', '--no-dirty', check: false).stdout().strip())

cargo_bin = find_program('cargo', version : '>=1.64.0', required : true)
cargo_meson_sh = files(meson.project_source_root() + '/scripts/cargo.sh')
cargo_toml = files('Cargo.toml')

libvaccel_dep = dependency('vaccel')

opt_async = get_option('async')
opt_async_stream = get_option('async-stream')
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

  vaccel_rpc_client = custom_target('vaccel-rpc-client',
    capture : false,
    console : true,
    build_by_default : true,
    install : install_client,
    install_dir : [get_option('libdir'), get_option('includedir')],
    input : files('Cargo.toml'),
    depfile : 'libvaccel_rpc_client.d',
    output : ['libvaccel_rpc_client.a', 'vaccel-rpc-client.h'],
    env : {
      'RUSTFLAGS': '-C panic=abort',
      'CARGO_TARGET_DIR': join_paths(meson.current_build_dir(), 'cargo_target'),
      'PKG_CONFIG_PATH':
        join_paths(libvaccel_dep.get_variable(pkgconfig: 'libdir'), 'pkgconfig')
        + ':$PKG_CONFIG_PATH',
      },
    command : [cargo_meson_sh, cargo_bin, get_option('buildtype'),
      files('Cargo.toml'), '*vaccel*client.*', '@OUTDIR@',
      features, features_client, '-p', 'vaccel-rpc-client'])

  cc = meson.get_compiler('c')
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
  vaccel_rpc_agent = custom_target('vaccel-rpc-agent',
    capture : false,
    console : true,
    build_by_default : true,
    install : true,
    install_dir : get_option('bindir'),
    input : files('Cargo.toml'),
    depfile : 'vaccel-rpc-agent.d',
    output : ['vaccel-rpc-agent'],
    env : {
      'RUSTFLAGS': '-C panic=abort',
      'CARGO_TARGET_DIR': join_paths(meson.current_build_dir(), 'cargo_target'),
      'PKG_CONFIG_PATH':
        join_paths(libvaccel_dep.get_variable(pkgconfig: 'libdir'), 'pkgconfig')
        + ':$PKG_CONFIG_PATH',
      },
    command : [cargo_meson_sh, cargo_bin, get_option('buildtype'),
      files('Cargo.toml'), 'vaccel-rpc-agent', '@OUTDIR@', features,
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
    'scripts/dist.sh',
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
    'scripts/dist.sh',
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
