from conan import ConanFile
from conan.tools.build import can_run
import os

class TestPackageConan(ConanFile):
        name        = 'ansi_colours_example'
        version     = '1.2'
        author      = 'Michał Nazarewicz <mina86@mina86.com>'

        settings = 'os', 'compiler', 'build_type', 'arch'
        options = {'shared': [True, False], 'fPIC': [True, False]}
        default_options = {'shared': False, 'fPIC': True}

        exports_sources = 'convert.c',
        requires = 'ansi_colours/[^1.0]'

        def config_options(self):
                if self.settings.os == 'Windows':
                        del self.options.fPIC

        def layout(self):
                from conan.tools.layout import basic_layout
                basic_layout(self)

        def build(self):
                deps = self.dependencies['ansi_colours'].cpp_info
                include_path = deps.includedirs[0]
                lib_path = deps.libdirs[0]

                comps = self.conf.get('tools.build:compiler_executables', {})
                self.run(' '.join((
                        comps.get('c') or os.environ.get('CC') or 'gcc',
                        '-Wall', '-Werror',
                        os.path.join(self.source_folder, 'convert.c'),
                        f'-I{include_path}',
                        f'-L{lib_path}',
                        '-lansi_colours',
                        '-o', 'convert',
                )))

        def test(self):
            if can_run(self):
                    self.run(os.path.join(self.build_folder, 'convert'))
