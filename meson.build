project('µtopia',
	version: '0.1.0',
	meson_version: '>= 0.50.0',
	default_options: [ 'warning_level=2'],
)

dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1')

app_id = 'dev.sp1rit.Gtopia'
utopia_prefix = get_option('prefix')
utopia_bindir = join_paths(utopia_prefix, get_option('bindir'))
utopia_localedir = join_paths(utopia_prefix, get_option('localedir'))
utopia_datadir = join_paths(utopia_prefix, get_option('datadir'))
utopia_pkgdatadir = join_paths(utopia_datadir, meson.project_name())


cargo_sources = files(
	'Cargo.toml',
	'Cargo.lock',
)

subdir('src')
subdir('data')

# Postinstall script (Update icon cache, gsettings, ...)
meson.add_install_script('build-aux/meson/postinstall.py')
