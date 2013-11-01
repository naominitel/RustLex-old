PARSER_LIB = lib/regex_parser.dylib
PARSER_C = src/rustlex/regex_parser.c
PARSER_SRC = $(PARSER_C:.c=.y)

# FIXME: fix build with clang
CC = gcc

RLDIR = src/rustlex
RUSTLEX_SRC = $(RLDIR)/lib.rs $(RLDIR)/regex.rs $(RLDIR)/dfa.rs \
			  $(RLDIR)/nfa.rs $(RLDIR)/automata.rs $(RLDIR)/regex.rs \
			  $(RLDIR)/action.rs

all: rustlex_lib

rustlex_lib: $(RUSTLEX_SRC) $(PARSER_LIB)
	rustpkg install rustlex

rustlex_tst: rustlex_lib

$(PARSER_LIB): $(PARSER_C)
	mkdir -p lib/
	$(CC) -shared -o $@ $^

$(PARSER_C): $(PARSER_SRC)
	yacc -o $@ $^ 

clean:
	rm -f $(PARSER_LIB)
	rm -f $(PARSER_C)
	rustpkg clean rustlex
	rm -rf lib/ build/ bin/
