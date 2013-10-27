%{
    #include <assert.h>
    #include <stdio.h>
    #include <stdlib.h>

    static char *yytext;

    static int current_pos = 0;

    typedef enum _rlex_ast_type
    {
        RustlexOr = 0,
        RustlexCat = 1,
        RustlexClos = 2,
        RustlexConst = 3
    } rustlex_ast_type_t;

    typedef struct _rlex_ast
    {
        rustlex_ast_type_t type;

        /* avoid unions for Rust FFI */
        struct _rlex_ast *op_left;
        struct _rlex_ast *op_right;
        char const_c;
        unsigned int const_pos;
    } rustlex_ast_t;

    static inline rustlex_ast_t* makeast(rustlex_ast_type_t t)
    {
        rustlex_ast_t *ret = malloc(sizeof(rustlex_ast_t));
        assert(ret != NULL);

        ret->type = t;
        return ret;
    }

    static inline rustlex_ast_t* makeor(rustlex_ast_t *l, rustlex_ast_t *r)
    {
        rustlex_ast_t *ret = makeast(RustlexOr);
        ret->op_left = l;
        ret->op_right = r;
        return ret;
    }

    static inline rustlex_ast_t* makecat(rustlex_ast_t *l, rustlex_ast_t *r)
    {
        rustlex_ast_t *ret = makeast(RustlexCat);
        ret->op_left = l;
        ret->op_right = r;
        return ret;
    }

    static inline rustlex_ast_t* makeclos(rustlex_ast_t *e)
    {
        rustlex_ast_t *ret = makeast(RustlexClos);
        ret->op_left = e;
        return ret;
    }

    static inline rustlex_ast_t* makeconst(char c)
    {
        rustlex_ast_t *ret = makeast(RustlexConst);
        ret->const_c = c;
        ret->const_pos = (current_pos ++);
        return ret;
    }

    static inline rustlex_ast_t* copyast(rustlex_ast_t *a)
    {
        rustlex_ast_t *ret = makeast(a->type);

        if(a->type == RustlexOr || a->type == RustlexCat)
        {
            ret->op_left = copyast(a->op_left);
            ret->op_right = copyast(a->op_right);
        }

        else if(a->type == RustlexClos)
            ret->op_left = copyast(a->op_left);

        else
            ret->const_c = a->const_c;

        return ret;
    }
    
%}

%union { rustlex_ast_t *ast; char ch; }
%token <ch> C
%type <ast> OR_EXPR CAT_EXPR CLOS_EXPR CONST_EXPR REGEX
%start REGEX

%%

REGEX 
    : OR_EXPR { $$ = $1; yylval.ast = $$; return; }
    ;

OR_EXPR 
    : OR_EXPR '|' CAT_EXPR { $$ = makeor($1, $3); }
    | CAT_EXPR             { $$ = $1; }
    ;

CAT_EXPR
    : CAT_EXPR CLOS_EXPR { $$ = makecat($1, $2); }
    | CLOS_EXPR          { $$ = $1; }
    ;

CLOS_EXPR
    : CONST_EXPR '*' { $$ = makeclos($1); }
    | CONST_EXPR '+' { $$ = makecat(copyast($1), makeclos($1)); }
    | CONST_EXPR     { $$ = $1; }
    ;

CONST_EXPR
    : '(' OR_EXPR ')' { $$ = $2; }
    | C               { $$ = makeconst(yytext[0]); }
    ;

%% 

int yyerror(char *s)
{
    fprintf(stderr, "Syntax error: %s", s);
    exit(EXIT_FAILURE);
}

char *input;

int yylex()
{
    if(*input == '\0')
    {
        /* end of input buffer */
        return EOF;
    }

    yytext = input;
    char tok = *(input ++);
    
    switch(tok)
    {
        case '|':
        case '*':
        case '(':
        case ')':
            return tok;

        default:
            break;
    }

    return C;
}

rustlex_ast_t* rustlex_parse_regex(char *in_tokens)
{
    input = in_tokens;
    current_pos = 0;
    yyparse();

    return yylval.ast;
}
