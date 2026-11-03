grammar LOLCODE;

//parser rules
lolcode         : HAI comments head variable_define body KBYE EOF ;

comments        : comment* ;
comment         : OBTW text TLDR ;

head            : ( MAEK_HEAD comments title MKAY comments )? ;
title           : GIMMEH_TITLE text OIC comments ;

body            : ( body_element comments )* ;
body_element    : paragraph
                | bold
                | list
                | link
                | newline
                | variable_use
                | WORD
                ;

paragraph       : MAEK_PARAGRAF comments variable_define inner_paragraph MKAY ;
inner_paragraph : ( paragraph_element comments )* ;
paragraph_element
                : bold
                | list
                | link
                | newline
                | variable_use
                | WORD
                ;

bold            : GIMMEH_BOLD variable_define inner_text OIC ;
inner_text      : ( variable_use | WORD )* ;

list            : MAEK_LIST variable_define list_items MKAY ;
list_items      : ( list_item comments )+ ;
list_item       : GIMMEH_ITEM variable_define inner_list OIC ;
inner_list      : ( bold | variable_use | WORD )* ;

link            : GIMMEH_LINX WORD OIC ;
newline         : NEWLINE ;

variable_define : ( IHAZ WORD ITIZ text MKAY comments )* ;
variable_use    : GIMMEH_VAR WORD OIC ;

text            : WORD+ ;


//lexical rules
HAI             : '#' H A I ;
KBYE            : '#' K B Y E ;
OBTW            : '#' O B T W ;
TLDR            : '#' T L D R ;
MAEK_HEAD       : '#' M A E K SEP H E A D ;
MAEK_PARAGRAF   : '#' M A E K SEP P A R A G R A F ;
MAEK_LIST       : '#' M A E K SEP L I S T ;
MKAY            : '#' M K A Y ;
GIMMEH_TITLE    : '#' G I M M E H SEP T I T L E ;
GIMMEH_BOLD     : '#' G I M M E H SEP B O L D ;
GIMMEH_ITEM     : '#' G I M M E H SEP I T E M ;
GIMMEH_LINX     : '#' G I M M E H SEP L I N X ;
GIMMEH_VAR      : '#' G I M M E H SEP V A R ;
OIC             : '#' O I C ;
NEWLINE         : '#' N E W L I N E ;
IHAZ            : '#' I H A Z ;
ITIZ            : '#' I T I Z ;

WORD 	:	 ('a'..'z' | 'A' .. 'Z' | '0'..'9'
                  | ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/' )+ ;

WS              : ( ' ' | '\t' | '\r' | '\n' )+ { skip(); } ;

fragment SEP    : ( ' ' | '\t' | '\r' | '\n' )+ ;

fragment A : 'a' | 'A' ;   fragment B : 'b' | 'B' ;   fragment C : 'c' | 'C' ;
fragment D : 'd' | 'D' ;   fragment E : 'e' | 'E' ;   fragment F : 'f' | 'F' ;
fragment G : 'g' | 'G' ;   fragment H : 'h' | 'H' ;   fragment I : 'i' | 'I' ;
fragment K : 'k' | 'K' ;   fragment L : 'l' | 'L' ;   fragment M : 'm' | 'M' ;
fragment N : 'n' | 'N' ;   fragment O : 'o' | 'O' ;   fragment P : 'p' | 'P' ;
fragment R : 'r' | 'R' ;   fragment S : 's' | 'S' ;   fragment T : 'T' | 't' ;
fragment V : 'v' | 'V' ;   fragment W : 'w' | 'W' ;   fragment X : 'x' | 'X' ;
fragment Y : 'y' | 'Y' ;   fragment Z : 'z' | 'Z' ;
