styles = []

scss_files = files([])

sassc = find_program('sassc', required: false)
if sassc.found()
	sassc_opts = [ '-a', '-M', '-t', 'expanded' ]
	styles += custom_target('Utopia',
		input: 'utopia.scss',
		output: 'utopia.css',
		command: [
		  sassc, sassc_opts, '@INPUT@', '@OUTPUT@',
		],
		depend_files: scss_files,
		build_by_default: true)
	install_data('utopia.scss', install_dir : utopia_pkgdatadir)
endif

gnome = import('gnome')

gtkresources = gnome.compile_resources('gtopia', 'gtopia.gresource.xml',
  dependencies: styles,
  gresource_bundle: true,
  install: true,
  install_dir: utopia_pkgdatadir
)

desktop_conf = configuration_data()
desktop_conf.set('icon', app_id)
configure_file(
    input: '@0@.desktop.in'.format(app_id),
    output: '@0@.desktop'.format(app_id),
    configuration: desktop_conf,
    install: true,
    install_dir: join_paths(utopia_datadir, 'applications')
)

install_data('@0@.svg'.format(app_id), install_dir : get_option('datadir')/'icons/hicolor/scalable/apps')
install_data('@0@.png'.format(app_id), install_dir : get_option('datadir')/'icons/hicolor/128x128/apps')

install_data('@0@-symbolic.svg'.format(app_id), install_dir : get_option('datadir')/'icons/hicolor/symbolic/apps')
