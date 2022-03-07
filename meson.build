project('gcribbage', 'c')
gnome = import('gnome')
gtkdeps = dependency('gtk4')
resources = gnome.compile_resources('gcribbage_resources', 'gcribbage_resources.xml', source_dir: 'src')
sources = ['gcribbage.c', 'application.c', 'window.c', 'table.c', 'simulation.c', 'hitbox.c', resources]
executable('gcribbage', sources: sources, dependencies: gtkdeps)
