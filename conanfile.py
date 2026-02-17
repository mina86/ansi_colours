import os

import conan


class AnsiColoursConan(conan.ConanFile):
        name        = 'ansi_colours'
        description = ('24-bit True Colour ↔ 256-colour ANSI terminal palette '
                       'conversion library')
        version     = '1.2.3'
        author      = 'Michał Nazarewicz <mina86@mina86.com>'
        license     = 'LGPL-3.0-or-later'
        url         = 'https://github.com/mina86/ansi_colours'
        topics      = "ansi", "terminal", "color", "rgb"

        settings = 'os', 'compiler', 'build_type', 'arch'
        options = {'shared': [True, False], 'fPIC': [True, False]}
        default_options = {'shared': False, 'fPIC': True}

        def config_options(self):
                if self.settings.os == 'Windows':
                        del self.options.fPIC

        exports_sources = 'LICENSE', 'src/*.[ch]'

        def layout(self):
                from conan.tools.layout import basic_layout
                basic_layout(self)

        def build(self):
                src_dir = os.path.join(self.source_folder, 'src')
                src = os.path.join(src_dir, 'ansi256.c')
                lib = 'libansi_colours.a'
                obj = 'ansi256.o'

                # Honestly I don’t know what I’m doing.  Should arguments be
                # shell-quoted?  Should I even use cflags?  Is cflags a list or
                # just a string?  Nevertheless, this seems to work.
                comps = self.conf.get('tools.build:compiler_executables', {})
                cc = comps.get('c') or os.environ.get('CC') or 'gcc'
                cflags = self.conf.get('tools.build:cflags',
                                       default='-Wall -Werror')
                if not isinstance(cflags, str):
                        cflags = ' '.join(cflags)
                self.run(f'{cc} {cflags} -I{src_dir} -c {src} -o {obj}')

                ar = os.environ.get('AR') or 'ar'
                self.run(f'{ar} rcs {lib} {obj}')

        def package(self):
                from conan.tools.files import copy

                copy(self, 'ansi_colours.h',
                     src=os.path.join(self.source_folder, 'src'),
                     dst=os.path.join(self.package_folder, 'include'),
                     keep_path=False)
                copy(self, '*.a',
                     src=self.build_folder,
                     dst=os.path.join(self.package_folder, 'lib'),
                     keep_path=False)

        def package_info(self):
            self.cpp_info.libs = ['ansi_colours']
