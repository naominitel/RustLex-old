PARSER_LIB = src/rustlex/regex_parser.dylib
PARSER_C = $(PARSER_LIB:.dylib=.c)
PARSER_SRC = $(PARSER_C:.c=.y)

$(PARSER_LIB): $(PARSER_C)
	gcc -g3 -shared -o $@ $^

$(PARSER_C): $(PARSER_SRC)
	yacc -o $@ $^ 


