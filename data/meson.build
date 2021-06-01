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


