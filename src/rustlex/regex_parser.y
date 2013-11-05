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

    static inline rustlex_ast_t* makeseq(unsigned char c1, unsigned char c2)
    {
        rustlex_ast_t *ret;
        unsigned char c;

        if(c1 >= c2)
           yyerror("Bad caracters for seq");

        ret = makeconst(c1);

        for(c = c1 + 1; c < c2; ++c)
        {
            rustlex_ast_t *op = makeconst(c);
            ret = makeor(ret, op);
        }

        ret = makeor(ret, makeconst(c2));

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
%start       REGEX
%token       TOK_OR TOK_CLOS TOK_OP TOK_CL TOK_PLUS TOK_ANY
%token       TOK_LBR TOK_RBR TOK_HYP
%token <ch>  C
%type  <ast> OR_EXPR CAT_EXPR CLOS_EXPR CONST_EXPR REGEX CLASS_EXPR SEQ CLASS
%type  <ast> ANY_EXPR

%%

REGEX 
    : OR_EXPR { $$ = $1; yylval.ast = $$; return; }
    ;

OR_EXPR 
    : OR_EXPR TOK_OR CAT_EXPR { $$ = makeor($1, $3); }
    | CAT_EXPR                { $$ = $1; }
    ;

CAT_EXPR
    : CAT_EXPR CLOS_EXPR { $$ = makecat($1, $2); }
    | CLOS_EXPR          { $$ = $1; }
    ;

CLOS_EXPR
    : ANY_EXPR TOK_CLOS { $$ = makeclos($1); }
    | ANY_EXPR TOK_PLUS { $$ = makecat(copyast($1), makeclos($1)); }
    | ANY_EXPR          { $$ = $1; }
    ;

ANY_EXPR
    : TOK_ANY           { $$ = makeseq(0, 255); }
    | CLASS_EXPR        { $$ = $1; }
    ;

CLASS_EXPR
    : TOK_LBR CLASS TOK_RBR { $$ = $2; }
    | CONST_EXPR            { $$ = $1; }
    ;

CLASS
    : SEQ CLASS     { $$ = makeor($1, $2); }
    | C CLASS       { $$ = makeor(makeconst($1), $2); }
    | SEQ           { $$ = $1; }
    | C             { $$ = makeconst($1); }
    ;

SEQ
    : C TOK_HYP C   { $$ = makeseq($1, $3); }
    ;

CONST_EXPR
    : TOK_OP OR_EXPR TOK_CL { $$ = $2; }
    | C                     { $$ = makeconst($1); }
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
            return TOK_OR;

        case '*':
            return TOK_CLOS;

        case '(':
            return TOK_OP;

        case ')':
            return TOK_CL;

        case '[':
            return TOK_LBR;

        case ']':
            return TOK_RBR;

        case '.':
            return TOK_ANY;

        case '+':
            return TOK_PLUS;

        case '-':
            return TOK_HYP;

        case '\\':
            yytext = input;
            input ++;

        default:
            break;
    }

    yylval.ch = yytext[0];
    return C;
}

rustlex_ast_t* rustlex_parse_regex(char *in_tokens)
{
    input = in_tokens;
    current_pos = 0;
    yyparse();

    return yylval.ast;
}
