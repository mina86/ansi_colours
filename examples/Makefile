# Example Makefile for building a simple program using the ansi_colours library.

CFLAGS = -Wall -Werror -O2


convert: objs/convert.o objs/ansi256.o
	$(CC) $(LDFLAGS) -o $@ $^

objs/ansi256.o: ../src/ansi256.c ../src/ansi_colours.h
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -I ../src -o $@ $<

objs/convert.o: convert.c ../src/ansi_colours.h
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -I ../src -o $@ $<


clean:
	rm -rf objs

distclean: clean
	rm -f convert
