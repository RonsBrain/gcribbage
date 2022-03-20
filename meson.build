project('gcribbage', 'c')
gnome = import('gnome')
gtkdeps = dependency('gtk4')
resources = gnome.compile_resources('gcribbage_resources', 'gcribbage_resources.xml', source_dir: 'src')
sources = [
    'gcribbage.c',
    'application.c',
    'window.c',
    'table.c',
    'simulation.c',
    'hitbox.c',
    'drawing.c',
    'scenes.c',
    'cards.c',
    'scoring.c',
    resources
]
executable('gcribbage', sources: sources, dependencies: gtkdeps)
test_score_pegging = executable('test_scoring', 'test/test_scoring.c', 'scoring.c', 'cards.c')
test('Test pegging scoring', test_score_pegging)
