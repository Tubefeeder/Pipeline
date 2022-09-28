global_conf = configuration_data()
global_conf.set('APP_ID', application_id)
global_conf.set('PKGDATADIR', pkgdatadir)
global_conf.set('PROFILE', profile)
global_conf.set('VERSION', version + version_suffix)
global_conf.set('GETTEXT_PACKAGE', gettext_package)
global_conf.set('LOCALEDIR', localedir)
global_conf.set('BUILD_DIR', meson.project_build_root())
config = configure_file(
  input: 'about.ui.in',
  output: 'about.ui',
  configuration: global_conf
)
run_command(
  'cp',
  meson.project_build_root() / 'data' / 'resources' / 'ui' / 'about.ui',
  meson.project_source_root() / 'data' / 'resources' / 'ui' / 'about.ui',
  check: true
)
