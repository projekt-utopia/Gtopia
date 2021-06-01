conf = configuration_data()
conf.set_quoted('APP_ID', 'dev.sp1rit.Gtopia')
conf.set_quoted('PKGDATADIR', utopia_pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

rust_sources = files(
	'main.rs',
	'application.rs',
	'config.rs',
	'grid/mod.rs',
	'grid/card.rs',
	'integration_item.rs',
	'utopia.rs',
	'uev/mod.rs',
	'uev/stream.rs'
)

sources = [
	cargo_sources,
	rust_sources
]

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
	'cargo-build',
	build_by_default: true,
	input: sources,
	output: meson.project_name(),
	console: true,
	install: true,
	install_dir: utopia_bindir,
	command: [
		cargo_script,
		meson.build_root(),
		meson.source_root(),
		'@OUTPUT@',
		get_option('buildtype'),
		meson.project_name(),
	]
)

